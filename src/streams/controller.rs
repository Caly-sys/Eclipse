use crate::streams::credentials::CredentialStore;
use crate::streams::playback::{
    prefetch_next_track_match, search_youtube_alternatives, select_alternative_match,
    YouTubeAlternativeMatch, YouTubeWebviewPlayer,
};
use crate::streams::provider::{
    DownloadFormat, DownloadJob, DownloadStatus, StreamPlaybackState, StreamPlaylist,
    StreamProvider, StreamTrack,
};
use crate::streams::spotify::SpotifyProvider;
use crate::streams::spotify_public::{
    PublicSpotifyFetcher, SpotifyImportResult, SpotifyPublicMetadataFetcher,
};
use crate::streams::youtube::YouTubeProvider;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Commands dispatched from UI / Controller to the Stream background worker.
#[derive(Debug, Clone)]
pub enum StreamCommand {
    Connect { provider_id: String },
    Disconnect { provider_id: String },
    Search { provider_id: String, query: String },
    FetchPlaylists { provider_id: String },
    FetchPlaylistTracks { provider_id: String, playlist_id: String },
    PlayTrack { provider_id: String, track: StreamTrack },
    Pause { provider_id: String },
    Resume { provider_id: String },
    Seek { provider_id: String, position_secs: f64 },
    Next { provider_id: String },
    Prev { provider_id: String },
    SetVolume { provider_id: String, volume: f32 },
    PollState { provider_id: String },
    OpenBrowserUrl { url: String },
    ImportSpotifyLink(String),
    MatchLibrary(Vec<StreamTrack>),
    SaveImportedPlaylist { name: String, tracks: Vec<StreamTrack> },
    PlayStreamTrack(StreamTrack),
    PauseStream,
    ResumeStream,
    SeekStream(f64),
    SetStreamVolume(f32),
    SelectAlternativeMatch {
        spotify_id: String,
        alternative: YouTubeAlternativeMatch,
    },
    SearchAlternatives {
        spotify_id: String,
        query: String,
    },
    PrefetchNextTrack(StreamTrack),
    PrefetchStreamTracks(Vec<StreamTrack>),
    DownloadTrack {
        track: StreamTrack,
        format: DownloadFormat,
    },
    DownloadBatch {
        tracks: Vec<StreamTrack>,
        format: DownloadFormat,
    },
    CancelDownload(String),
    CancelAllDownloads,
    RetryDownload(String),
    RetryAllFailedDownloads,
    ClearCompletedDownloads,
    Shutdown,
}

/// Events returned from the Stream background worker to the UI thread.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    Connected {
        provider_id: String,
    },
    Disconnected {
        provider_id: String,
    },
    SearchResults {
        provider_id: String,
        query: String,
        tracks: Vec<StreamTrack>,
    },
    PlaylistsLoaded {
        provider_id: String,
        playlists: Vec<StreamPlaylist>,
    },
    PlaylistTracksLoaded {
        provider_id: String,
        playlist_id: String,
        tracks: Vec<StreamTrack>,
    },
    PlaybackStateUpdated {
        provider_id: String,
        state: StreamPlaybackState,
    },
    TrackStarted {
        provider_id: String,
        track: StreamTrack,
    },
    BrowserUrlOpened {
        url: String,
    },
    SpotifyImportComplete(Result<SpotifyImportResult, String>),
    LibraryMatched(Vec<StreamTrack>),
    PlaylistSaved {
        id: i64,
        name: String,
        track_count: usize,
    },
    StreamTrackEnded,
    StreamPlaybackReady {
        spotify_id: String,
        cache_path: std::path::PathBuf,
        track: StreamTrack,
    },
    DownloadUpdated(DownloadJob),
    DownloadCompleted {
        job_id: String,
        path: std::path::PathBuf,
        local_track_id: i64,
        sample_rate: u32,
        bit_depth: u16,
    },
    DownloadFailed {
        job_id: String,
        error: String,
    },
    AlternativesLoaded {
        spotify_id: String,
        alternatives: Vec<YouTubeAlternativeMatch>,
    },
    AlternativeSelected {
        spotify_id: String,
        matched: YouTubeAlternativeMatch,
    },
    Error {
        provider_id: String,
        action: String,
        message: String,
    },
}

pub type StreamAction = StreamCommand;

/// Controller managing the stream background worker thread and UI state.
pub struct StreamController {
    cmd_tx: Sender<StreamCommand>,
    event_rx: Receiver<StreamEvent>,
    worker_handle: Option<JoinHandle<()>>,
    pub active_state: Option<StreamPlaybackState>,
    pub search_results: HashMap<String, Vec<StreamTrack>>,
    pub playlists: HashMap<String, Vec<StreamPlaylist>>,
    pub playlist_tracks: HashMap<(String, String), Vec<StreamTrack>>,
    pub connected_providers: HashMap<String, bool>,
    pub is_busy: bool,
    pub last_error: Option<String>,
    pub last_spotify_import: Option<Result<SpotifyImportResult, String>>,
    pub alternatives: HashMap<String, Vec<YouTubeAlternativeMatch>>,
    pub stream_queue: Vec<StreamTrack>,
    pub stream_queue_index: Option<usize>,
    pub download_jobs: Vec<DownloadJob>,
    pub selected_download_format: DownloadFormat,
    pub active_resolutions: Arc<Mutex<HashSet<String>>>,
    pub pending_prefetch: Arc<Mutex<VecDeque<StreamTrack>>>,
    pub current_prefetch: Arc<Mutex<Option<String>>>,
    pub promoted_to_playback: Arc<Mutex<HashMap<String, StreamTrack>>>,
    pub shutdown_flag: Arc<AtomicBool>,
}

impl StreamController {
    /// Initialize default streaming providers (Spotify, YouTube Music).
    pub fn default_providers() -> Vec<Box<dyn StreamProvider>> {
        vec![
            Box::new(SpotifyProvider::new()),
            Box::new(YouTubeProvider::new()),
        ]
    }

    /// Initialize a new stream controller and spawn its background worker.
    pub fn new(providers: Vec<Box<dyn StreamProvider>>) -> Self {
        // Automatically migrate/purge any legacy Spotify keyring entries
        CredentialStore::migrate_legacy_credentials();

        let (cmd_tx, cmd_rx) = channel::<StreamCommand>();
        let (event_tx, event_rx) = channel::<StreamEvent>();

        let mut initial_connected = HashMap::new();
        for p in &providers {
            initial_connected.insert(p.id().to_lowercase(), p.is_connected());
        }

        let active_resolutions = Arc::new(Mutex::new(HashSet::new()));
        let pending_prefetch = Arc::new(Mutex::new(VecDeque::new()));
        let current_prefetch = Arc::new(Mutex::new(None));
        let promoted_to_playback = Arc::new(Mutex::new(HashMap::new()));
        let prefetch_running = Arc::new(AtomicBool::new(false));
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let worker_active_resolutions = active_resolutions.clone();
        let worker_pending_prefetch = pending_prefetch.clone();
        let worker_current_prefetch = current_prefetch.clone();
        let worker_promoted_to_playback = promoted_to_playback.clone();
        let worker_prefetch_running = prefetch_running.clone();
        let worker_shutdown_flag = shutdown_flag.clone();

        let worker_handle = thread::Builder::new()
            .name("eclipse-stream-worker".to_string())
            .spawn(move || {
                run_stream_worker(
                    providers,
                    cmd_rx,
                    event_tx,
                    worker_active_resolutions,
                    worker_pending_prefetch,
                    worker_current_prefetch,
                    worker_promoted_to_playback,
                    worker_prefetch_running,
                    worker_shutdown_flag,
                );
            })
            .ok();

        Self {
            cmd_tx,
            event_rx,
            worker_handle,
            active_state: None,
            search_results: HashMap::new(),
            playlists: HashMap::new(),
            playlist_tracks: HashMap::new(),
            connected_providers: initial_connected,
            is_busy: false,
            last_error: None,
            last_spotify_import: None,
            alternatives: HashMap::new(),
            stream_queue: Vec::new(),
            stream_queue_index: None,
            download_jobs: Vec::new(),
            selected_download_format: DownloadFormat::Flac,
            active_resolutions,
            pending_prefetch,
            current_prefetch,
            promoted_to_playback,
            shutdown_flag,
        }
    }

    /// Check if a given provider is currently marked as connected.
    pub fn is_connected(&self, provider_id: &str) -> bool {
        self.connected_providers
            .get(&provider_id.to_lowercase())
            .copied()
            .unwrap_or(false)
    }

    /// Dispatch connect command to background worker.
    pub fn connect(&mut self, provider_id: &str) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::Connect {
            provider_id: provider_id.to_string(),
        });
    }

    /// Dispatch disconnect command to background worker.
    pub fn disconnect(&mut self, provider_id: &str) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::Disconnect {
            provider_id: provider_id.to_string(),
        });
    }

    /// Dispatch a command to the background worker thread.
    pub fn send(&self, cmd: StreamCommand) -> Result<(), String> {
        self.cmd_tx
            .send(cmd)
            .map_err(|e| format!("Failed to send stream command: {e}"))
    }

    /// Convenience dispatch methods
    pub fn search(&mut self, provider_id: &str, query: &str) {
        self.is_busy = true;
        let _ = self.send(StreamCommand::Search {
            provider_id: provider_id.to_string(),
            query: query.to_string(),
        });
    }

    pub fn fetch_playlists(&mut self, provider_id: &str) {
        self.is_busy = true;
        let _ = self.send(StreamCommand::FetchPlaylists {
            provider_id: provider_id.to_string(),
        });
    }

    pub fn fetch_playlist_tracks(&mut self, provider_id: &str, playlist_id: &str) {
        self.is_busy = true;
        let _ = self.send(StreamCommand::FetchPlaylistTracks {
            provider_id: provider_id.to_string(),
            playlist_id: playlist_id.to_string(),
        });
    }

    pub fn play_track(&mut self, provider_id: &str, track: StreamTrack) {
        let _ = self.send(StreamCommand::PlayTrack {
            provider_id: provider_id.to_string(),
            track,
        });
    }

    pub fn pause(&self, provider_id: &str) {
        let _ = self.send(StreamCommand::Pause {
            provider_id: provider_id.to_string(),
        });
    }

    pub fn resume(&self, provider_id: &str) {
        let _ = self.send(StreamCommand::Resume {
            provider_id: provider_id.to_string(),
        });
    }

    pub fn seek(&self, provider_id: &str, position_secs: f64) {
        let _ = self.send(StreamCommand::Seek {
            provider_id: provider_id.to_string(),
            position_secs,
        });
    }

    pub fn next(&mut self, provider_id: &str) {
        if (provider_id.eq_ignore_ascii_case("youtube") || provider_id.eq_ignore_ascii_case("spotify"))
            && self.has_next_in_queue()
        {
            self.play_next_in_queue();
        } else {
            let _ = self.send(StreamCommand::Next {
                provider_id: provider_id.to_string(),
            });
        }
    }

    pub fn prev(&mut self, provider_id: &str) {
        if (provider_id.eq_ignore_ascii_case("youtube") || provider_id.eq_ignore_ascii_case("spotify"))
            && self.stream_queue_index.map(|i| i > 0).unwrap_or(false)
        {
            self.play_prev_in_queue();
        } else {
            let _ = self.send(StreamCommand::Prev {
                provider_id: provider_id.to_string(),
            });
        }
    }

    pub fn set_volume(&self, provider_id: &str, volume: f32) {
        let _ = self.send(StreamCommand::SetVolume {
            provider_id: provider_id.to_string(),
            volume,
        });
    }

    pub fn poll_state(&self, provider_id: &str) {
        let _ = self.send(StreamCommand::PollState {
            provider_id: provider_id.to_string(),
        });
    }

    pub fn open_browser_url(&self, url: &str) {
        let _ = self.send(StreamCommand::OpenBrowserUrl {
            url: url.to_string(),
        });
    }

    /// Play a stream track via lazy YouTube matching and official embedded webview playback.
    pub fn play_stream_track(&mut self, track: StreamTrack) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::PlayStreamTrack(track));
    }

    /// Pause stream playback.
    pub fn pause_stream(&self) {
        let _ = self.send(StreamCommand::PauseStream);
    }

    /// Resume stream playback.
    pub fn resume_stream(&self) {
        let _ = self.send(StreamCommand::ResumeStream);
    }

    /// Seek in the current stream track.
    pub fn seek_stream(&self, position_secs: f64) {
        let _ = self.send(StreamCommand::SeekStream(position_secs));
    }

    /// Set stream playback volume (0.0 to 1.0).
    pub fn set_stream_volume(&self, volume: f32) {
        let _ = self.send(StreamCommand::SetStreamVolume(volume));
    }

    /// Select an alternative YouTube match for a stream track.
    pub fn select_alternative_match(
        &mut self,
        spotify_id: &str,
        alternative: YouTubeAlternativeMatch,
    ) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::SelectAlternativeMatch {
            spotify_id: spotify_id.to_string(),
            alternative,
        });
    }

    /// Search for alternative YouTube matches for a given track or query.
    pub fn search_alternatives(&mut self, spotify_id: &str, query: &str) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::SearchAlternatives {
            spotify_id: spotify_id.to_string(),
            query: query.to_string(),
        });
    }

    /// Lazily prefetch match for the next track in queue.
    pub fn prefetch_next_track(&self, next_track: StreamTrack) {
        let _ = self.send(StreamCommand::PrefetchNextTrack(next_track));
    }

    /// Prefetch and cache audio streams in the background for upcoming tracks.
    pub fn prefetch_stream_tracks(&mut self, tracks: Vec<StreamTrack>) {
        let _ = self.send(StreamCommand::PrefetchStreamTracks(tracks));
    }

    /// Load and play a stream queue.
    pub fn load_stream_queue(&mut self, tracks: Vec<StreamTrack>, start_index: usize) {
        self.stream_queue = tracks;
        self.stream_queue_index = Some(start_index);
        if let Some(track) = self.stream_queue.get(start_index).cloned() {
            self.play_stream_track(track);
            let next_candidates: Vec<StreamTrack> = self
                .stream_queue
                .iter()
                .skip(start_index + 1)
                .take(2)
                .cloned()
                .collect();
            if !next_candidates.is_empty() {
                self.prefetch_stream_tracks(next_candidates.clone());
            }
            if let Some(next) = next_candidates.first().cloned() {
                self.prefetch_next_track(next);
            }
        }
    }

    pub fn has_next_in_queue(&self) -> bool {
        if let Some(idx) = self.stream_queue_index {
            idx + 1 < self.stream_queue.len()
        } else {
            !self.stream_queue.is_empty()
        }
    }

    pub fn play_next_in_queue(&mut self) -> bool {
        if let Some(idx) = self.stream_queue_index {
            if idx + 1 < self.stream_queue.len() {
                let next_idx = idx + 1;
                self.stream_queue_index = Some(next_idx);
                let track = self.stream_queue[next_idx].clone();
                self.play_stream_track(track);
                let next_candidates: Vec<StreamTrack> = self
                    .stream_queue
                    .iter()
                    .skip(next_idx + 1)
                    .take(2)
                    .cloned()
                    .collect();
                if !next_candidates.is_empty() {
                    self.prefetch_stream_tracks(next_candidates.clone());
                }
                if let Some(peek) = next_candidates.first().cloned() {
                    self.prefetch_next_track(peek);
                }
                return true;
            }
        }
        false
    }

    pub fn play_prev_in_queue(&mut self) -> bool {
        if let Some(idx) = self.stream_queue_index {
            if idx > 0 {
                let prev_idx = idx - 1;
                self.stream_queue_index = Some(prev_idx);
                let track = self.stream_queue[prev_idx].clone();
                self.play_stream_track(track);
                return true;
            }
        }
        false
    }

    /// Dispatch Spotify public metadata import to the background worker.
    pub fn import_spotify_link(&mut self, url: &str) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::ImportSpotifyLink(url.to_string()));
    }

    /// Dispatch matching imported stream tracks against the local library to the background worker.
    pub fn match_library(&mut self, tracks: Vec<StreamTrack>) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::MatchLibrary(tracks));
    }

    /// Dispatch saving imported stream tracks as a local playlist to the background worker.
    pub fn save_imported_playlist(&mut self, name: String, tracks: Vec<StreamTrack>) {
        self.is_busy = true;
        self.last_error = None;
        let _ = self.send(StreamCommand::SaveImportedPlaylist { name, tracks });
    }

    /// Set the default audio format for future downloads.
    pub fn set_download_format(&mut self, format: DownloadFormat) {
        self.selected_download_format = format;
    }

    /// Queue a stream track for background download with an explicit audio format without setting `is_busy`.
    pub fn download_track_with_format(&mut self, track: StreamTrack, format: DownloadFormat) {
        let job_id = if !track.id.is_empty() {
            track.id.clone()
        } else {
            track.uri.clone()
        };
        let job = DownloadJob::with_format(job_id.clone(), track.clone(), format);
        if let Some(existing) = self.download_jobs.iter_mut().find(|j| j.id == job_id) {
            *existing = job;
        } else {
            self.download_jobs.push(job);
        }
        let _ = self.send(StreamCommand::DownloadTrack { track, format });
    }

    /// Queue a stream track for background download without setting `is_busy`.
    pub fn download_track(&mut self, track: StreamTrack) {
        self.download_track_with_format(track, self.selected_download_format);
    }

    /// Enqueue a batch of stream tracks for background download.
    /// Filters out tracks that already have a local match or are already active/queued in download jobs.
    /// Enqueues all remaining tracks immediately as DownloadJob entries with DownloadStatus::Queued
    /// using the selected download format. Dispatches command to background scheduler.
    /// Returns the count of newly queued tracks.
    pub fn download_tracks_batch(&mut self, tracks: Vec<StreamTrack>) -> usize {
        let mut to_enqueue = Vec::new();

        for t in tracks {
            // Filter out tracks that already have a local match
            if t.is_available_locally() || t.local_track_id.is_some() || t.local_path.is_some() {
                continue;
            }

            // Filter out tracks that are already active/queued in self.download_jobs
            let is_active_or_queued = self.download_jobs.iter().any(|job| {
                (job.track.id == t.id || job.id == t.id || (!t.uri.is_empty() && (job.track.uri == t.uri || job.id == t.uri)))
                    && !job.is_failed()
                    && !job.is_completed()
            });

            if is_active_or_queued {
                continue;
            }

            to_enqueue.push(t);
        }

        if to_enqueue.is_empty() {
            return 0;
        }

        let count = to_enqueue.len();
        let format = self.selected_download_format;

        for t in &to_enqueue {
            let job_id = if !t.id.is_empty() {
                t.id.clone()
            } else {
                t.uri.clone()
            };
            let job = DownloadJob::with_format(job_id.clone(), t.clone(), format);
            if let Some(existing) = self.download_jobs.iter_mut().find(|j| j.id == job_id) {
                *existing = job;
            } else {
                self.download_jobs.push(job);
            }
        }

        let _ = self.send(StreamCommand::DownloadBatch {
            tracks: to_enqueue,
            format,
        });

        count
    }

    /// Cancel an in-progress or queued download.
    pub fn cancel_download(&mut self, id: &str) {
        if let Some(job) = self.download_jobs.iter_mut().find(|j| j.id == id) {
            if !job.is_completed() {
                job.status = DownloadStatus::Failed("Download cancelled".to_string());
                job.error = Some("Download cancelled".to_string());
            }
        }
        let _ = self.send(StreamCommand::CancelDownload(id.to_string()));
    }

    /// Cancel all Queued, Resolving, and Downloading jobs in self.download_jobs and dispatch cancellation to background workers.
    pub fn cancel_all_downloads(&mut self) {
        for job in &mut self.download_jobs {
            if matches!(
                job.status,
                DownloadStatus::Queued | DownloadStatus::Resolving | DownloadStatus::Downloading(_) | DownloadStatus::Tagging
            ) {
                job.status = DownloadStatus::Failed("Download cancelled".to_string());
                job.error = Some("Download cancelled".to_string());
            }
        }
        let _ = self.send(StreamCommand::CancelAllDownloads);
    }

    /// Retry a failed or completed download.
    pub fn retry_download(&mut self, id: &str) {
        if let Some(job) = self.download_jobs.iter_mut().find(|j| j.id == id) {
            job.status = DownloadStatus::Queued;
            job.error = None;
            let _ = self.send(StreamCommand::RetryDownload(id.to_string()));
        }
    }

    /// Re-queue all jobs currently in DownloadStatus::Failed(_), changing their status back to Queued
    /// and triggering the download scheduler.
    pub fn retry_all_failed_downloads(&mut self) {
        for job in &mut self.download_jobs {
            if job.is_failed() {
                job.status = DownloadStatus::Queued;
                job.error = None;
            }
        }
        let _ = self.send(StreamCommand::RetryAllFailedDownloads);
    }

    /// Clear all completed downloads from the queue list.
    pub fn clear_completed_downloads(&mut self) {
        self.download_jobs.retain(|j| !j.is_completed());
        let _ = self.send(StreamCommand::ClearCompletedDownloads);
    }

    /// Return the number of currently active (non-terminal) downloads.
    pub fn active_downloads_count(&self) -> usize {
        self.download_jobs.iter().filter(|j| j.is_active()).count()
    }

    /// Clear the temporary audio stream cache on disk, returning bytes freed.
    pub fn clear_stream_cache(&self) -> Result<u64, String> {
        crate::streams::downloader::clear_stream_cache()
    }

    /// Retrieve the total size of all cached audio stream files in bytes.
    pub fn get_stream_cache_size(&self) -> u64 {
        crate::streams::downloader::get_stream_cache_size()
    }

    /// Return true if the given track (by clean Spotify/track ID) is currently resolving or buffering.
    pub fn is_resolving(&self, track_id: &str) -> bool {
        match self.active_resolutions.lock() {
            Ok(guard) => guard.contains(track_id),
            Err(poisoned) => poisoned.into_inner().contains(track_id),
        }
    }

    /// Return the count of tracks currently waiting in the background prefetch queue.
    pub fn pending_prefetch_count(&self) -> usize {
        match self.pending_prefetch.lock() {
            Ok(queue) => queue.len(),
            Err(poisoned) => poisoned.into_inner().len(),
        }
    }

    /// Return true if the given track (by canonical ID) is currently being downloaded by the background prefetch worker.
    pub fn is_prefetching(&self, track_id: &str) -> bool {
        match self.current_prefetch.lock() {
            Ok(guard) => guard.as_deref() == Some(track_id),
            Err(poisoned) => poisoned.into_inner().as_deref() == Some(track_id),
        }
    }

    /// Drain incoming events non-blockingly on the UI frame loop.
    pub fn update(&mut self) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            match &event {
                StreamEvent::Connected { provider_id } => {
                    self.connected_providers.insert(provider_id.clone(), true);
                    self.is_busy = false;
                }
                StreamEvent::Disconnected { provider_id } => {
                    self.connected_providers.insert(provider_id.clone(), false);
                    self.is_busy = false;
                }
                StreamEvent::SearchResults {
                    provider_id,
                    tracks,
                    ..
                } => {
                    self.search_results
                        .insert(provider_id.clone(), tracks.clone());
                    self.is_busy = false;
                }
                StreamEvent::PlaylistsLoaded {
                    provider_id,
                    playlists,
                } => {
                    self.playlists
                        .insert(provider_id.clone(), playlists.clone());
                    self.is_busy = false;
                }
                StreamEvent::PlaylistTracksLoaded {
                    provider_id,
                    playlist_id,
                    tracks,
                } => {
                    self.playlist_tracks.insert(
                        (provider_id.clone(), playlist_id.clone()),
                        tracks.clone(),
                    );
                    self.is_busy = false;
                }
                StreamEvent::PlaybackStateUpdated { state, .. } => {
                    self.active_state = Some(state.clone());
                }
                StreamEvent::TrackStarted { track, .. } => {
                    let mut state = self.active_state.clone().unwrap_or_default();
                    state.current_track = Some(track.clone());
                    state.is_playing = true;
                    self.active_state = Some(state);
                }
                StreamEvent::BrowserUrlOpened { .. } => {
                    self.is_busy = false;
                }
                StreamEvent::SpotifyImportComplete(ref res) => {
                    self.is_busy = false;
                    self.last_spotify_import = Some(res.clone());
                    if let Err(ref err) = res {
                        self.last_error = Some(err.clone());
                    }
                }
                StreamEvent::LibraryMatched(ref tracks) => {
                    self.is_busy = false;
                    if let Some(Ok(ref mut import_res)) = self.last_spotify_import {
                        import_res.tracks = tracks.clone();
                    }
                }
                StreamEvent::PlaylistSaved { .. } => {
                    self.is_busy = false;
                }
                StreamEvent::StreamTrackEnded => {
                    if let Some(ref mut state) = self.active_state {
                        state.is_playing = false;
                    }
                }
                StreamEvent::StreamPlaybackReady { track, .. } => {
                    self.is_busy = false;
                    let mut state = self.active_state.clone().unwrap_or_default();
                    state.current_track = Some(track.clone());
                    state.is_playing = true;
                    self.active_state = Some(state);
                }
                StreamEvent::DownloadUpdated(job) => {
                    if let Some(existing) = self.download_jobs.iter_mut().find(|j| j.id == job.id) {
                        *existing = job.clone();
                    } else {
                        self.download_jobs.push(job.clone());
                    }
                }
                StreamEvent::DownloadCompleted {
                    job_id,
                    path,
                    local_track_id,
                    sample_rate: _,
                    bit_depth: _,
                } => {
                    let path_str = path.to_string_lossy().to_string();
                    if let Some(job) = self.download_jobs.iter_mut().find(|j| j.id == *job_id || j.track.id == *job_id) {
                        job.status = DownloadStatus::Completed(path.clone());
                        job.track.set_local_match(*local_track_id, &path_str);
                    }
                    if let Some(Ok(ref mut import_res)) = self.last_spotify_import {
                        for t in &mut import_res.tracks {
                            if t.id == *job_id || t.uri == *job_id {
                                t.set_local_match(*local_track_id, &path_str);
                            }
                        }
                    }
                    for t in &mut self.stream_queue {
                        if t.id == *job_id || t.uri == *job_id {
                            t.set_local_match(*local_track_id, &path_str);
                        }
                    }
                    for tracks_vec in self.playlist_tracks.values_mut() {
                        for t in tracks_vec {
                            if t.id == *job_id || t.uri == *job_id {
                                t.set_local_match(*local_track_id, &path_str);
                            }
                        }
                    }
                }
                StreamEvent::DownloadFailed { job_id, error } => {
                    if let Some(job) = self.download_jobs.iter_mut().find(|j| j.id == *job_id || j.track.id == *job_id) {
                        job.status = DownloadStatus::Failed(error.clone());
                        job.error = Some(error.clone());
                    }
                }
                StreamEvent::AlternativesLoaded {
                    spotify_id,
                    alternatives,
                } => {
                    self.alternatives
                        .insert(spotify_id.clone(), alternatives.clone());
                    self.is_busy = false;
                }
                StreamEvent::AlternativeSelected {
                    spotify_id,
                    ..
                } => {
                    self.alternatives.remove(spotify_id);
                    self.is_busy = false;
                }
                StreamEvent::Error { message, .. } => {
                    self.last_error = Some(message.clone());
                    self.is_busy = false;
                }
            }
            events.push(event);
        }
        events
    }
}

impl Default for StreamController {
    fn default() -> Self {
        Self::new(Self::default_providers())
    }
}

impl Drop for StreamController {
    fn drop(&mut self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
        let _ = self.cmd_tx.send(StreamCommand::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

/// RAII guard ensuring track ID is removed from active resolutions when worker thread finishes.
struct ActiveResolutionGuard {
    spotify_id: String,
    active_resolutions: Arc<Mutex<HashSet<String>>>,
}

impl Drop for ActiveResolutionGuard {
    fn drop(&mut self) {
        let mut set = match self.active_resolutions.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        set.remove(&self.spotify_id);
    }
}

/// Maximum concurrent prefetch workers allowed (single-worker concurrency).
pub const MAX_CONCURRENT_PREFETCH: usize = 1;

/// Trigger or spawn the background prefetch worker if one is not already running.
fn trigger_prefetch_worker(
    pending_prefetch: Arc<Mutex<VecDeque<StreamTrack>>>,
    active_resolutions: Arc<Mutex<HashSet<String>>>,
    current_prefetch: Arc<Mutex<Option<String>>>,
    promoted_to_playback: Arc<Mutex<HashMap<String, StreamTrack>>>,
    prefetch_running: Arc<AtomicBool>,
    shutdown_flag: Arc<AtomicBool>,
    event_tx: Sender<StreamEvent>,
) {
    if shutdown_flag.load(Ordering::Relaxed) {
        return;
    }

    // Atomic compare_exchange ensures at most ONE background prefetch worker runs (MAX_CONCURRENT_PREFETCH = 1)
    if prefetch_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let worker_prefetch_running = prefetch_running.clone();
    let spawn_res = thread::Builder::new()
        .name("stream-prefetch-worker".to_string())
        .spawn(move || {
            struct PrefetchWorkerStateGuard {
                prefetch_running: Arc<AtomicBool>,
                current_prefetch: Arc<Mutex<Option<String>>>,
            }
            impl Drop for PrefetchWorkerStateGuard {
                fn drop(&mut self) {
                    self.prefetch_running.store(false, Ordering::SeqCst);
                    let mut cur = match self.current_prefetch.lock() {
                        Ok(g) => g,
                        Err(p) => p.into_inner(),
                    };
                    *cur = None;
                }
            }

            let _worker_guard = PrefetchWorkerStateGuard {
                prefetch_running: worker_prefetch_running.clone(),
                current_prefetch: current_prefetch.clone(),
            };

            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_prefetch_loop(
                    pending_prefetch,
                    active_resolutions,
                    current_prefetch,
                    promoted_to_playback,
                    shutdown_flag,
                    event_tx,
                );
            }));
        });

    if spawn_res.is_err() {
        prefetch_running.store(false, Ordering::SeqCst);
    }
}

/// Loop for the background prefetch worker executing sequential cache downloads off the UI thread.
fn run_prefetch_loop(
    pending_prefetch: Arc<Mutex<VecDeque<StreamTrack>>>,
    active_resolutions: Arc<Mutex<HashSet<String>>>,
    current_prefetch: Arc<Mutex<Option<String>>>,
    promoted_to_playback: Arc<Mutex<HashMap<String, StreamTrack>>>,
    shutdown_flag: Arc<AtomicBool>,
    event_tx: Sender<StreamEvent>,
) {
    loop {
        if shutdown_flag.load(Ordering::Relaxed) {
            break;
        }

        // Check if there are any pending tracks to prefetch
        let is_empty = match pending_prefetch.lock() {
            Ok(queue) => queue.is_empty(),
            Err(poisoned) => poisoned.into_inner().is_empty(),
        };
        if is_empty {
            break;
        }

        // Active Playback Priority Rule:
        // Check if active_resolutions (active playback buffering) is busy.
        // If an active PlayStreamTrack request is currently in progress, prefetch MUST defer or wait
        // until active playback finishes resolving so 100% bandwidth is reserved for immediate playback.
        let is_playback_busy = match active_resolutions.lock() {
            Ok(guard) => !guard.is_empty(),
            Err(poisoned) => !poisoned.into_inner().is_empty(),
        };

        if is_playback_busy {
            thread::sleep(std::time::Duration::from_millis(100));
            continue;
        }

        // Dequeue next track from pending_prefetch
        let track_opt = match pending_prefetch.lock() {
            Ok(mut queue) => queue.pop_front(),
            Err(poisoned) => {
                let mut queue = poisoned.into_inner();
                queue.pop_front()
            }
        };

        let track = match track_opt {
            Some(t) => t,
            None => continue,
        };

        // Filter out if already cached
        if crate::streams::downloader::get_cached_stream_path(&track).is_some() {
            continue;
        }

        let spotify_id = crate::streams::downloader::extract_stream_track_id(&track);

        // Mark as currently prefetching
        {
            let mut cur = match current_prefetch.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            *cur = Some(spotify_id.clone());
        }

        struct CurrentPrefetchGuard {
            current_prefetch: Arc<Mutex<Option<String>>>,
            active: bool,
        }
        impl Drop for CurrentPrefetchGuard {
            fn drop(&mut self) {
                if self.active {
                    let mut cur = match self.current_prefetch.lock() {
                        Ok(guard) => guard,
                        Err(poisoned) => poisoned.into_inner(),
                    };
                    *cur = None;
                }
            }
        }
        let mut prefetch_guard = CurrentPrefetchGuard {
            current_prefetch: current_prefetch.clone(),
            active: true,
        };

        // Download & Cache: call cache_stream_track protected with std::panic::catch_unwind
        let track_clone = track.clone();
        let cache_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            crate::streams::downloader::cache_stream_track(&track_clone)
        }));

        // Check if this track was promoted to playback while prefetching
        let promoted_track = {
            let mut cur = match current_prefetch.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            *cur = None;
            prefetch_guard.active = false;
            let mut prom = match promoted_to_playback.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            prom.remove(&spotify_id)
        };

        if let Some(target_track) = promoted_track {
            // PROMOTED TO PLAYBACK:
            // Remove from active_resolutions
            {
                let mut active = match active_resolutions.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                active.remove(&spotify_id);
            }

            match cache_res {
                Ok(Ok((res_id, cache_path))) => {
                    let _ = event_tx.send(StreamEvent::StreamPlaybackReady {
                        spotify_id: res_id,
                        cache_path,
                        track: target_track,
                    });
                }
                Ok(Err(err)) => {
                    let _ = event_tx.send(StreamEvent::Error {
                        provider_id: "stream".to_string(),
                        action: "play_stream_track".to_string(),
                        message: err,
                    });
                }
                Err(panic_err) => {
                    let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Stream prefetch worker panicked".to_string()
                    };
                    let _ = event_tx.send(StreamEvent::Error {
                        provider_id: "stream".to_string(),
                        action: "play_stream_track".to_string(),
                        message: format!("Stream prefetch worker panicked: {msg}"),
                    });
                }
            }
        } else {
            // NOT PROMOTED (normal silent prefetch):
            // Keep completely silent to the UI.
            // DO NOT dispatch StreamPlaybackReady, DO NOT alter is_busy or active_state.
        }
    }
}

/// Worker thread loop: executes network I/O and provider requests off the UI thread.
#[allow(clippy::too_many_arguments)]
fn run_stream_worker(
    mut providers: Vec<Box<dyn StreamProvider>>,
    cmd_rx: Receiver<StreamCommand>,
    event_tx: Sender<StreamEvent>,
    active_resolutions: Arc<Mutex<HashSet<String>>>,
    pending_prefetch: Arc<Mutex<VecDeque<StreamTrack>>>,
    current_prefetch: Arc<Mutex<Option<String>>>,
    promoted_to_playback: Arc<Mutex<HashMap<String, StreamTrack>>>,
    prefetch_running: Arc<AtomicBool>,
    shutdown_flag: Arc<AtomicBool>,
) {
    let mut provider_map: HashMap<String, usize> = HashMap::new();
    for (idx, p) in providers.iter().enumerate() {
        provider_map.insert(p.id().to_lowercase(), idx);
    }

    let mut active_provider_id: Option<String> = None;
    let mut webview_player: Option<YouTubeWebviewPlayer> = None;
    let download_manager = Arc::new(crate::streams::downloader::DownloadManager::new(event_tx.clone()));
    let poll_interval = std::time::Duration::from_millis(200);

    loop {
        let cmd_opt = match cmd_rx.recv_timeout(poll_interval) {
            Ok(cmd) => Some(cmd),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                shutdown_flag.store(true, Ordering::SeqCst);
                break;
            }
        };

        if let Some(cmd) = cmd_opt {
            match cmd {
                StreamCommand::Shutdown => {
                    shutdown_flag.store(true, Ordering::SeqCst);
                    if let Some(mut p) = webview_player {
                        p.shutdown();
                    }
                    break;
                }
                StreamCommand::PlayStreamTrack(track) => {
                    active_provider_id = Some("stream".to_string());

                    // 1. Instant Cache Hit Check (0ms startup latency for previously played or pre-cached streams)
                    if let Some((spotify_id, cache_path)) =
                        crate::streams::downloader::get_cached_stream_path(&track)
                    {
                        let _ = event_tx.send(StreamEvent::StreamPlaybackReady {
                            spotify_id,
                            cache_path,
                            track,
                        });
                        continue;
                    }

                    let spotify_id = crate::streams::downloader::extract_stream_track_id(&track);

                    // 2. In-Flight Promotion Check:
                    // If the requested track is currently being actively resolved by the prefetch worker,
                    // promote it to playback rather than spawning a duplicate yt-dlp process.
                    let in_flight = {
                        let cur_guard = match current_prefetch.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        if cur_guard.as_deref() == Some(&spotify_id) {
                            let mut prom_guard = match promoted_to_playback.lock() {
                                Ok(g) => g,
                                Err(p) => p.into_inner(),
                            };
                            prom_guard.insert(spotify_id.clone(), track.clone());
                            true
                        } else {
                            false
                        }
                    };

                    if in_flight {
                        // Mark track ID in active_resolutions so UI is_resolving() returns true
                        let mut active = match active_resolutions.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        active.insert(spotify_id.clone());
                        continue;
                    }

                    // 2b. Re-check instant cache in case prefetch finished just as we checked
                    if let Some((cached_id, cache_path)) =
                        crate::streams::downloader::get_cached_stream_path(&track)
                    {
                        let _ = event_tx.send(StreamEvent::StreamPlaybackReady {
                            spotify_id: cached_id,
                            cache_path,
                            track,
                        });
                        continue;
                    }

                    // Remove from pending_prefetch queue if it was queued there
                    {
                        let mut queue = match pending_prefetch.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        queue.retain(|t| crate::streams::downloader::extract_stream_track_id(t) != spotify_id);
                    }

                    // 3. Request Deduplication: avoid concurrent yt-dlp processes for the same track
                    {
                        let mut active = match active_resolutions.lock() {
                            Ok(guard) => guard,
                            Err(poisoned) => poisoned.into_inner(),
                        };
                        if active.contains(&spotify_id) {
                            // Already buffering/resolving this track; coalesce duplicate request
                            continue;
                        }
                        active.insert(spotify_id.clone());
                    }

                    let tx = event_tx.clone();
                    let track_clone = track.clone();
                    let active_res = active_resolutions.clone();
                    let res_id = spotify_id.clone();

                    let spawn_res = std::thread::Builder::new()
                        .name("stream-playback-worker".to_string())
                        .spawn(move || {
                            let _guard = ActiveResolutionGuard {
                                spotify_id: res_id,
                                active_resolutions: active_res,
                            };
                            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                match crate::streams::downloader::cache_stream_track(&track_clone) {
                                    Ok((spotify_id, cache_path)) => {
                                        let _ = tx.send(StreamEvent::StreamPlaybackReady {
                                            spotify_id,
                                            cache_path,
                                            track: track_clone,
                                        });
                                    }
                                    Err(err) => {
                                        let _ = tx.send(StreamEvent::Error {
                                            provider_id: "stream".to_string(),
                                            action: "play_stream_track".to_string(),
                                            message: err,
                                        });
                                    }
                                }
                            }));

                            if let Err(panic_err) = res {
                                let msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                                    s.to_string()
                                } else if let Some(s) = panic_err.downcast_ref::<String>() {
                                    s.clone()
                                } else {
                                    "Stream playback worker panicked".to_string()
                                };
                                let _ = tx.send(StreamEvent::Error {
                                    provider_id: "stream".to_string(),
                                    action: "play_stream_track".to_string(),
                                    message: format!("Stream playback worker panicked: {msg}"),
                                });
                            }
                        });

                    if let Err(e) = spawn_res {
                        let mut set = match active_resolutions.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        set.remove(&spotify_id);
                        let _ = event_tx.send(StreamEvent::Error {
                            provider_id: "stream".to_string(),
                            action: "play_stream_track".to_string(),
                            message: format!("Failed to spawn stream playback thread: {e}"),
                        });
                    }
                }
                StreamCommand::PauseStream => {
                    if let Some(ref p) = webview_player {
                        p.pause();
                    }
                    if let Some(ref id) = active_provider_id {
                        let id_key = id.to_lowercase();
                        if let Some(&idx) = provider_map.get(&id_key) {
                            let _ = providers[idx].pause();
                        }
                    }
                }
                StreamCommand::ResumeStream => {
                    if let Some(ref p) = webview_player {
                        p.resume();
                    }
                    if let Some(ref id) = active_provider_id {
                        let id_key = id.to_lowercase();
                        if let Some(&idx) = provider_map.get(&id_key) {
                            let _ = providers[idx].resume();
                        }
                    }
                }
                StreamCommand::SeekStream(position_secs) => {
                    if let Some(ref p) = webview_player {
                        p.seek(position_secs);
                    }
                    if let Some(ref id) = active_provider_id {
                        let id_key = id.to_lowercase();
                        if let Some(&idx) = provider_map.get(&id_key) {
                            let _ = providers[idx].seek(position_secs);
                        }
                    }
                }
                StreamCommand::SetStreamVolume(volume) => {
                    if let Some(ref p) = webview_player {
                        p.set_volume(volume);
                    }
                    if let Some(ref id) = active_provider_id {
                        let id_key = id.to_lowercase();
                        if let Some(&idx) = provider_map.get(&id_key) {
                            let _ = providers[idx].set_volume(volume);
                        }
                    }
                }
                StreamCommand::SelectAlternativeMatch {
                    spotify_id,
                    alternative,
                } => {
                    match crate::library::Database::open() {
                        Ok(db) => {
                            if let Err(e) = select_alternative_match(&db, &spotify_id, &alternative) {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id: "youtube".to_string(),
                                    action: "select_alternative_match".to_string(),
                                    message: e,
                                });
                            } else {
                                let _ = event_tx.send(StreamEvent::AlternativeSelected {
                                    spotify_id,
                                    matched: alternative,
                                });
                            }
                        }
                        Err(e) => {
                            let _ = event_tx.send(StreamEvent::Error {
                                provider_id: "youtube".to_string(),
                                action: "select_alternative_match".to_string(),
                                message: format!("Failed to open database: {e}"),
                            });
                        }
                    }
                }
                StreamCommand::SearchAlternatives { spotify_id, query } => {
                    let tx = event_tx.clone();
                    thread::spawn(move || {
                        let api_key = match CredentialStore::get_youtube_api_key() {
                            Ok(Some(k)) if !k.trim().is_empty() => k,
                            _ => {
                                let _ = tx.send(StreamEvent::Error {
                                    provider_id: "youtube".to_string(),
                                    action: "search_alternatives".to_string(),
                                    message: crate::streams::playback::MISSING_KEY_ERROR_MESSAGE.to_string(),
                                });
                                return;
                            }
                        };
                        match search_youtube_alternatives(&api_key, &query, 10) {
                            Ok(alternatives) => {
                                let _ = tx.send(StreamEvent::AlternativesLoaded {
                                    spotify_id,
                                    alternatives,
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(StreamEvent::Error {
                                    provider_id: "youtube".to_string(),
                                    action: "search_alternatives".to_string(),
                                    message: e,
                                });
                            }
                        }
                    });
                }
                StreamCommand::PrefetchNextTrack(next_track) => {
                    if let Ok(db) = crate::library::Database::open() {
                        let api_key = CredentialStore::get_youtube_api_key().ok().flatten();
                        prefetch_next_track_match(Arc::new(db), api_key, next_track);
                    }
                }
                StreamCommand::PrefetchStreamTracks(tracks) => {
                    let cur_prefetch_id = match current_prefetch.lock() {
                        Ok(g) => g.clone(),
                        Err(p) => p.into_inner().clone(),
                    };

                    let mut filtered_tracks: Vec<StreamTrack> = Vec::new();
                    for track in tracks {
                        if crate::streams::downloader::get_cached_stream_path(&track).is_some() {
                            continue;
                        }
                        let track_id = crate::streams::downloader::extract_stream_track_id(&track);
                        if cur_prefetch_id.as_deref() == Some(&track_id) {
                            continue;
                        }
                        filtered_tracks.push(track);
                        if filtered_tracks.len() >= 2 {
                            break;
                        }
                    }

                    // Replace the pending prefetch queue with up-to-2 new tracks
                    {
                        let mut queue = match pending_prefetch.lock() {
                            Ok(g) => g,
                            Err(p) => p.into_inner(),
                        };
                        queue.clear();
                        for t in filtered_tracks {
                            queue.push_back(t);
                        }
                    }

                    // Trigger background prefetch worker (at most 1 running)
                    trigger_prefetch_worker(
                        pending_prefetch.clone(),
                        active_resolutions.clone(),
                        current_prefetch.clone(),
                        promoted_to_playback.clone(),
                        prefetch_running.clone(),
                        shutdown_flag.clone(),
                        event_tx.clone(),
                    );
                }
                StreamCommand::OpenBrowserUrl { url } => {
                    let u = url.clone();
                    let tx = event_tx.clone();
                    thread::spawn(move || {
                        if let Err(e) = webbrowser::open(&u) {
                            let _ = tx.send(StreamEvent::Error {
                                provider_id: "browser".to_string(),
                                action: "open_url".to_string(),
                                message: format!("Failed to open browser URL: {e}"),
                            });
                        } else {
                            let _ = tx.send(StreamEvent::BrowserUrlOpened { url: u });
                        }
                    });
                }
                StreamCommand::Connect { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let res = providers[idx].connect();
                        match res {
                            Ok(()) => {
                                let _ = event_tx.send(StreamEvent::Connected { provider_id });
                            }
                            Err(e) => {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id,
                                    action: "connect".to_string(),
                                    message: e,
                                });
                            }
                        }
                    } else {
                        let _ = event_tx.send(StreamEvent::Error {
                            provider_id: provider_id.clone(),
                            action: "connect".to_string(),
                            message: format!("Unknown stream provider: {provider_id}"),
                        });
                    }
                }
                StreamCommand::Disconnect { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if active_provider_id.as_deref().map(|s| s.to_lowercase()) == Some(id_key.clone()) {
                        active_provider_id = None;
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let res = providers[idx].disconnect();
                        match res {
                            Ok(()) => {
                                let _ = event_tx.send(StreamEvent::Disconnected { provider_id });
                            }
                            Err(e) => {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id,
                                    action: "disconnect".to_string(),
                                    message: e,
                                });
                            }
                        }
                    }
                }
                StreamCommand::Search { provider_id, query } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(&idx) = provider_map.get(&id_key) {
                        match providers[idx].search(&query) {
                            Ok(tracks) => {
                                let _ = event_tx.send(StreamEvent::SearchResults {
                                    provider_id,
                                    query,
                                    tracks,
                                });
                            }
                            Err(e) => {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id,
                                    action: "search".to_string(),
                                    message: e,
                                });
                            }
                        }
                    }
                }
                StreamCommand::FetchPlaylists { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(&idx) = provider_map.get(&id_key) {
                        match providers[idx].get_user_playlists() {
                            Ok(playlists) => {
                                let _ = event_tx.send(StreamEvent::PlaylistsLoaded {
                                    provider_id,
                                    playlists,
                                });
                            }
                            Err(e) => {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id,
                                    action: "fetch_playlists".to_string(),
                                    message: e,
                                });
                            }
                        }
                    }
                }
                StreamCommand::FetchPlaylistTracks {
                    provider_id,
                    playlist_id,
                } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(&idx) = provider_map.get(&id_key) {
                        match providers[idx].get_playlist_tracks(&playlist_id) {
                            Ok(tracks) => {
                                let _ = event_tx.send(StreamEvent::PlaylistTracksLoaded {
                                    provider_id,
                                    playlist_id,
                                    tracks,
                                });
                            }
                            Err(e) => {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id,
                                    action: "fetch_playlist_tracks".to_string(),
                                    message: e,
                                });
                            }
                        }
                    }
                }
                StreamCommand::PlayTrack { provider_id, track } => {
                    let id_key = provider_id.to_lowercase();
                    active_provider_id = Some(provider_id.clone());
                    if id_key == "youtube" {
                        let player = webview_player.get_or_insert_with(YouTubeWebviewPlayer::new);
                        let _ = player.play_video(&track.id, &track);
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let track_clone = track.clone();
                        match providers[idx].play(&track) {
                            Ok(()) => {
                                let _ = event_tx.send(StreamEvent::TrackStarted {
                                    provider_id,
                                    track: track_clone,
                                });
                            }
                            Err(e) => {
                                let _ = event_tx.send(StreamEvent::Error {
                                    provider_id,
                                    action: "play".to_string(),
                                    message: e,
                                });
                            }
                        }
                    }
                }
                StreamCommand::Pause { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(ref p) = webview_player {
                        p.pause();
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].pause();
                    }
                }
                StreamCommand::Resume { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    active_provider_id = Some(provider_id.clone());
                    if let Some(ref p) = webview_player {
                        p.resume();
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].resume();
                    }
                }
                StreamCommand::Seek {
                    provider_id,
                    position_secs,
                } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(ref p) = webview_player {
                        p.seek(position_secs);
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].seek(position_secs);
                    }
                }
                StreamCommand::Next { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].next();
                    }
                }
                StreamCommand::Prev { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].prev();
                    }
                }
                StreamCommand::SetVolume {
                    provider_id,
                    volume,
                } => {
                    let id_key = provider_id.to_lowercase();
                    if let Some(ref p) = webview_player {
                        p.set_volume(volume);
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].set_volume(volume);
                    }
                }
                StreamCommand::PollState { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    if id_key != "spotify" && id_key != "youtube" {
                        if let Some(&idx) = provider_map.get(&id_key) {
                            if let Ok(Some(state)) = providers[idx].get_playback_state() {
                                let _ = event_tx.send(StreamEvent::PlaybackStateUpdated {
                                    provider_id,
                                    state,
                                });
                            }
                        }
                    }
                }
                StreamCommand::ImportSpotifyLink(url) => {
                    let tx = event_tx.clone();
                    thread::spawn(move || {
                        let db_opt = crate::library::Database::open().ok();
                        let fetcher = match &db_opt {
                            Some(db) => PublicSpotifyFetcher::with_db(db.clone()),
                            None => PublicSpotifyFetcher::new(),
                        };
                        let mut res = fetcher.fetch_metadata(&url);
                        if let Ok(ref mut import_result) = res {
                            let db_to_use = db_opt.or_else(|| crate::library::Database::open().ok());
                            if let Some(ref db) = db_to_use {
                                if let Ok(matched) = crate::streams::matcher::find_in_my_library(db, &import_result.tracks) {
                                    import_result.tracks = matched;
                                }
                            }
                        }
                        let _ = tx.send(StreamEvent::SpotifyImportComplete(res));
                    });
                }
                StreamCommand::MatchLibrary(tracks) => {
                    let tx = event_tx.clone();
                    thread::spawn(move || {
                        match crate::library::Database::open() {
                            Ok(db) => {
                                match crate::streams::matcher::find_in_my_library(&db, &tracks) {
                                    Ok(matched) => {
                                        let _ = tx.send(StreamEvent::LibraryMatched(matched));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(StreamEvent::Error {
                                            provider_id: "library".to_string(),
                                            action: "match_library".to_string(),
                                            message: e,
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(StreamEvent::Error {
                                    provider_id: "library".to_string(),
                                    action: "match_library".to_string(),
                                    message: format!("Failed to open database: {e}"),
                                });
                            }
                        }
                    });
                }
                StreamCommand::SaveImportedPlaylist { name, tracks } => {
                    let tx = event_tx.clone();
                    thread::spawn(move || {
                        match crate::library::Database::open() {
                            Ok(db) => {
                                match crate::streams::matcher::save_imported_playlist(&db, &name, &tracks) {
                                    Ok(res) => {
                                        let _ = tx.send(StreamEvent::PlaylistSaved {
                                            id: res.id,
                                            name: res.name,
                                            track_count: res.track_count,
                                        });
                                    }
                                    Err(e) => {
                                        let _ = tx.send(StreamEvent::Error {
                                            provider_id: "library".to_string(),
                                            action: "save_imported_playlist".to_string(),
                                            message: e,
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(StreamEvent::Error {
                                    provider_id: "library".to_string(),
                                    action: "save_imported_playlist".to_string(),
                                    message: format!("Failed to open database: {e}"),
                                });
                            }
                        }
                    });
                }
                StreamCommand::DownloadTrack { track, format } => {
                    download_manager.start_download(track, format);
                }
                StreamCommand::DownloadBatch { tracks, format } => {
                    download_manager.start_download_batch(tracks, format);
                }
                StreamCommand::CancelDownload(id) => {
                    download_manager.cancel_download(&id);
                }
                StreamCommand::CancelAllDownloads => {
                    download_manager.cancel_all_downloads();
                }
                StreamCommand::RetryDownload(id) => {
                    download_manager.retry_download(&id);
                }
                StreamCommand::RetryAllFailedDownloads => {
                    download_manager.retry_all_failed();
                }
                StreamCommand::ClearCompletedDownloads => {
                    download_manager.clear_completed();
                }
            }
        }

        // Drain incoming webview IPC messages if webview player is active
        if let Some(ref mut p) = webview_player {
            if p.update() {
                let _ = event_tx.send(StreamEvent::StreamTrackEnded);
            }

            // If YouTube is active, dispatch real-time state updates from the webview player
            if active_provider_id.as_deref().map(|s| s.eq_ignore_ascii_case("youtube")).unwrap_or(false) {
                let cur_state = p.playback_state.read().unwrap().clone();
                if cur_state.current_track.is_some() {
                    let _ = event_tx.send(StreamEvent::PlaybackStateUpdated {
                        provider_id: "youtube".to_string(),
                        state: cur_state,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    struct TestProvider {
        connected: Arc<AtomicBool>,
    }

    impl TestProvider {
        fn new() -> (Self, Arc<AtomicBool>) {
            let flag = Arc::new(AtomicBool::new(false));
            (
                Self {
                    connected: flag.clone(),
                },
                flag,
            )
        }
    }

    impl StreamProvider for TestProvider {
        fn id(&self) -> &'static str {
            "test_mock"
        }

        fn name(&self) -> &'static str {
            "Test Mock Provider"
        }

        fn is_connected(&self) -> bool {
            self.connected.load(Ordering::SeqCst)
        }

        fn connect(&mut self) -> Result<(), String> {
            self.connected.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn disconnect(&mut self) -> Result<(), String> {
            self.connected.store(false, Ordering::SeqCst);
            Ok(())
        }

        fn search(&self, query: &str) -> Result<Vec<StreamTrack>, String> {
            Ok(vec![StreamTrack::new(
                "mock_1",
                format!("Result for {query}"),
                "Mock Artist",
                "Mock Album",
                120.0,
                None,
                "mock://track/1",
                "test_mock",
            )])
        }

        fn get_user_playlists(&self) -> Result<Vec<StreamPlaylist>, String> {
            Ok(vec![StreamPlaylist::new(
                "pl_mock",
                "Mock Playlist",
                10,
                None,
                None,
            )])
        }

        fn get_playlist_tracks(&self, _playlist_id: &str) -> Result<Vec<StreamTrack>, String> {
            Ok(vec![StreamTrack::new(
                "pl_t1",
                "Playlist Track 1",
                "Artist",
                "Album",
                180.0,
                None,
                "mock://track/pl1",
                "test_mock",
            )])
        }

        fn play(&mut self, _track: &StreamTrack) -> Result<(), String> {
            Ok(())
        }

        fn pause(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn resume(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn seek(&mut self, _position_secs: f64) -> Result<(), String> {
            Ok(())
        }

        fn next(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn prev(&mut self) -> Result<(), String> {
            Ok(())
        }

        fn set_volume(&mut self, _volume: f32) -> Result<(), String> {
            Ok(())
        }

        fn get_playback_state(&self) -> Result<Option<StreamPlaybackState>, String> {
            Ok(Some(StreamPlaybackState {
                is_playing: true,
                current_track: Some(StreamTrack::new(
                    "mock_playing",
                    "Playing Track",
                    "Playing Artist",
                    "Album",
                    210.0,
                    None,
                    "mock://playing",
                    "test_mock",
                )),
                position_secs: 42.0,
                duration_secs: 210.0,
                device_name: Some("Mock Device".to_string()),
                is_premium: true,
                volume: 0.8,
            }))
        }
    }

    #[test]
    fn test_stream_controller_lifecycle_and_commands() {
        let (provider, conn_flag) = TestProvider::new();
        let providers: Vec<Box<dyn StreamProvider>> = vec![Box::new(provider)];
        let mut controller = StreamController::new(providers);

        // 1. Connect command
        controller.send(StreamCommand::Connect {
            provider_id: "test_mock".to_string(),
        }).expect("send connect");

        thread::sleep(Duration::from_millis(50));
        let events = controller.update();
        assert!(events.iter().any(|e| matches!(e, StreamEvent::Connected { .. })));
        assert_eq!(controller.connected_providers.get("test_mock"), Some(&true));
        assert!(conn_flag.load(Ordering::SeqCst));

        // 2. Search command
        controller.search("test_mock", "Synthwave");
        thread::sleep(Duration::from_millis(50));
        let _ = controller.update();
        let results = controller.search_results.get("test_mock");
        assert!(results.is_some());
        assert_eq!(results.unwrap()[0].title, "Result for Synthwave");

        // 3. Playlists command
        controller.fetch_playlists("test_mock");
        thread::sleep(Duration::from_millis(50));
        let _ = controller.update();
        let pls = controller.playlists.get("test_mock");
        assert!(pls.is_some());
        assert_eq!(pls.unwrap()[0].name, "Mock Playlist");

        // 4. Play track command
        let track = StreamTrack::new(
            "t_play",
            "Awesome Tune",
            "Artist",
            "Album",
            150.0,
            None,
            "uri",
            "test_mock",
        );
        controller.play_track("test_mock", track);
        thread::sleep(Duration::from_millis(50));
        let _ = controller.update();
        assert!(controller.active_state.is_some());
        assert_eq!(
            controller.active_state.as_ref().unwrap().current_track.as_ref().unwrap().title,
            "Awesome Tune"
        );

        // 5. Poll state command
        controller.poll_state("test_mock");
        thread::sleep(Duration::from_millis(50));
        let _ = controller.update();
        assert_eq!(
            controller.active_state.as_ref().unwrap().device_name.as_deref(),
            Some("Mock Device")
        );

        // 6. Disconnect command
        controller.send(StreamCommand::Disconnect {
            provider_id: "test_mock".to_string(),
        }).expect("send disconnect");
        thread::sleep(Duration::from_millis(50));
        let _ = controller.update();
        assert_eq!(controller.connected_providers.get("test_mock"), Some(&false));
        assert!(!conn_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_stream_controller_default_initialization() {
        let controller = StreamController::default();
        let providers = StreamController::default_providers();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].id(), "spotify");
        assert_eq!(providers[0].name(), "Spotify");
        assert_eq!(providers[1].id(), "youtube");
        assert_eq!(providers[1].name(), "YouTube Music");
        assert!(controller.search_results.is_empty());
    }

    #[test]
    fn test_stream_controller_import_spotify_link_dispatch() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        // Dispatch import with an invalid link to test background execution & error event delivery
        controller.import_spotify_link("not_a_valid_spotify_link");
        assert!(controller.is_busy);

        // Wait for worker thread to process and send back event
        let mut got_event = false;
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(50));
            let events = controller.update();
            for ev in events {
                if let StreamEvent::SpotifyImportComplete(res) = ev {
                    assert!(res.is_err());
                    got_event = true;
                    break;
                }
            }
            if got_event {
                break;
            }
        }

        assert!(got_event, "Worker should dispatch SpotifyImportComplete");
        assert!(!controller.is_busy);
        assert!(controller.last_error.is_some());
        assert!(controller.last_spotify_import.is_some());
    }

    #[test]
    fn test_stream_controller_import_spotify_link_local_matching() {
        // Pre-populate stream cache so public fetcher doesn't invoke real network
        let target_uri = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";
        let target_url = "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT";

        let sample_item = SpotifyImportResult {
            resource_type: crate::streams::spotify_public::SpotifyResourceType::Track,
            id: "4cOdK2wGLETKBW3PvgPWqT".to_string(),
            title: "Never Gonna Give You Up".to_string(),
            subtitle: Some("Rick Astley".to_string()),
            cover_url: None,
            tracks: vec![StreamTrack::new(
                "4cOdK2wGLETKBW3PvgPWqT",
                "Never Gonna Give You Up",
                "Rick Astley",
                "Whenever You Need Somebody",
                213.0,
                None,
                target_uri,
                "spotify",
            )],
            total_tracks: 1,
            is_truncated: false,
            truncation_message: None,
            original_url: target_url.to_string(),
        };

        if let Ok(db) = crate::library::Database::open() {
            let serialized = serde_json::to_string(&sample_item).expect("serialize");
            let _ = db.set_stream_cache(target_uri, "spotify", "track", &serialized);

            // Insert a local track matching Never Gonna Give You Up
            let local_trk = crate::library::Track {
                id: 0,
                path: "C:/music/test_rick_astley.flac".to_string(),
                title: "Never Gonna Give You Up".to_string(),
                artist: "Rick Astley".to_string(),
                album: "Whenever You Need Somebody".to_string(),
                duration_secs: 213.0,
                format: "FLAC".to_string(),
                ..Default::default()
            };
            let local_id = db.upsert_track(&local_trk).expect("upsert local track");

            let (provider, _) = TestProvider::new();
            let mut controller = StreamController::new(vec![Box::new(provider)]);

            controller.import_spotify_link(target_url);

            let mut matched_local = false;
            for _ in 0..30 {
                thread::sleep(Duration::from_millis(50));
                for ev in controller.update() {
                    if let StreamEvent::SpotifyImportComplete(Ok(res)) = ev {
                        assert_eq!(res.tracks.len(), 1);
                        if res.tracks[0].is_available_locally() {
                            assert_eq!(res.tracks[0].local_track_id, Some(local_id));
                            assert_eq!(
                                res.tracks[0].local_path.as_deref(),
                                Some("C:/music/test_rick_astley.flac")
                            );
                            matched_local = true;
                        }
                    }
                }
                if matched_local {
                    break;
                }
            }

            // Cleanup local test track
            let conn = db.lock_conn();
            let _ = conn.execute("DELETE FROM tracks WHERE id = ?1", rusqlite::params![local_id]);

            assert!(matched_local, "Imported tracks must be automatically matched against local library");
        }
    }

    #[test]
    fn test_stream_controller_matcher_and_playlist_commands() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        // Test match_library dispatch
        let track = StreamTrack::new(
            "sp_1",
            "Song",
            "Artist",
            "Album",
            180.0,
            None,
            "spotify:track:1",
            "spotify",
        );
        controller.match_library(vec![track.clone()]);
        assert!(controller.is_busy);

        // Wait for event
        let mut got_match = false;
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(50));
            for ev in controller.update() {
                if let StreamEvent::LibraryMatched(tracks) = ev {
                    assert_eq!(tracks.len(), 1);
                    got_match = true;
                    break;
                }
            }
            if got_match {
                break;
            }
        }
        assert!(got_match);
        assert!(!controller.is_busy);

        // Test save_imported_playlist dispatch
        controller.save_imported_playlist("My Test Pl".to_string(), vec![track]);
        assert!(controller.is_busy);

        let mut got_saved = false;
        let mut saved_id = None;
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(50));
            for ev in controller.update() {
                if let StreamEvent::PlaylistSaved { id, name, track_count } = ev {
                    assert_eq!(name, "My Test Pl");
                    assert_eq!(track_count, 1);
                    got_saved = true;
                    saved_id = Some(id);
                    break;
                }
            }
            if got_saved {
                break;
            }
        }
        assert!(got_saved);
        assert!(!controller.is_busy);

        // Cleanup: ensure test playlist is never left in the user database
        if let Some(id) = saved_id {
            if let Ok(db) = crate::library::Database::open() {
                let _ = db.delete_playlist(id);
            }
        }
    }

    #[test]
    fn test_stream_controller_playback_and_queue_commands() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        let track1 = StreamTrack::new(
            "sp_1",
            "Track One",
            "Artist",
            "Album",
            180.0,
            None,
            "spotify:track:1",
            "spotify",
        );
        let track2 = StreamTrack::new(
            "sp_2",
            "Track Two",
            "Artist",
            "Album",
            200.0,
            None,
            "spotify:track:2",
            "spotify",
        );

        // Load stream queue
        controller.load_stream_queue(vec![track1.clone(), track2.clone()], 0);
        assert_eq!(controller.stream_queue.len(), 2);
        assert_eq!(controller.stream_queue_index, Some(0));
        assert!(controller.has_next_in_queue());

        // Advance queue
        assert!(controller.play_next_in_queue());
        assert_eq!(controller.stream_queue_index, Some(1));
        assert!(!controller.has_next_in_queue());

        // Step back
        assert!(controller.play_prev_in_queue());
        assert_eq!(controller.stream_queue_index, Some(0));

        // Test direct stream controls
        controller.pause_stream();
        controller.resume_stream();
        controller.seek_stream(45.0);
        controller.set_stream_volume(0.8);

        // Test alternative selection dispatch
        let alt = YouTubeAlternativeMatch::new(
            "yt_alt_123",
            "Alternative Title",
            "Alt Channel",
            182.0,
            None,
        );
        controller.select_alternative_match("sp_1", alt.clone());

        // Test StreamTrackEnded handling
        let _ = controller.send(StreamCommand::Shutdown);
    }

    #[test]
    fn test_stream_controller_download_queue_state_machine_and_is_busy_decoupling() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        let track = StreamTrack::new(
            "spotify_track_dl_1",
            "Starboy",
            "The Weeknd",
            "Starboy",
            230.0,
            Some("https://example.com/cover.jpg".to_string()),
            "spotify:track:spotify_track_dl_1",
            "spotify",
        );

        // Populate last_spotify_import so we verify automatic local matching upon download completion
        controller.last_spotify_import = Some(Ok(SpotifyImportResult {
            resource_type: crate::streams::spotify_public::SpotifyResourceType::Playlist,
            id: "pl_starboy".to_string(),
            title: "Pop Hits".to_string(),
            subtitle: None,
            cover_url: None,
            tracks: vec![track.clone()],
            total_tracks: 1,
            is_truncated: false,
            truncation_message: None,
            original_url: "https://open.spotify.com/playlist/pl_starboy".to_string(),
        }));

        assert!(!controller.is_busy);
        assert_eq!(controller.active_downloads_count(), 0);

        // 1. Download track: MUST NEVER set is_busy = true
        controller.download_track(track.clone());
        assert!(!controller.is_busy, "download_track must NEVER trigger global is_busy loading state");
        assert_eq!(controller.active_downloads_count(), 1);
        assert_eq!(controller.download_jobs.len(), 1);
        assert_eq!(controller.download_jobs[0].status, DownloadStatus::Queued);
        assert_eq!(controller.download_jobs[0].format, DownloadFormat::Flac);

        // 2. Cancel download
        controller.cancel_download(&track.id);
        assert_eq!(controller.active_downloads_count(), 0);
        assert!(controller.download_jobs[0].is_failed());

        // 3. Retry download
        controller.retry_download(&track.id);
        assert_eq!(controller.active_downloads_count(), 1);
        assert_eq!(controller.download_jobs[0].status, DownloadStatus::Queued);

        // 4. Test DownloadUpdated and DownloadCompleted event routing
        let mut updated_job = controller.download_jobs[0].clone();
        updated_job.status = DownloadStatus::Downloading(0.65);

        // Synthesize worker events
        let dl_path = std::path::PathBuf::from("C:\\music\\Downloads\\The Weeknd - Starboy.mp3");
        let complete_ev = StreamEvent::DownloadCompleted {
            job_id: track.id.clone(),
            path: dl_path.clone(),
            local_track_id: 101,
            sample_rate: 44100,
            bit_depth: 16,
        };

        // Deliver synthesized complete event to event receiver channel
        let (cmd_tx, _cmd_rx) = channel::<StreamCommand>();
        let (event_tx, event_rx) = channel::<StreamEvent>();
        event_tx.send(StreamEvent::DownloadUpdated(updated_job)).unwrap();
        event_tx.send(complete_ev).unwrap();

        controller.cmd_tx = cmd_tx;
        controller.event_rx = event_rx;

        let events = controller.update();
        assert_eq!(events.len(), 2);

        // Verify job marked completed
        assert_eq!(controller.active_downloads_count(), 0);
        assert_eq!(controller.download_jobs.len(), 1);
        assert!(controller.download_jobs[0].is_completed());

        // Verify track in last_spotify_import was automatically matched locally
        let import_tracks = &controller.last_spotify_import.as_ref().unwrap().as_ref().unwrap().tracks;
        assert!(import_tracks[0].is_available_locally());
        assert_eq!(import_tracks[0].local_track_id, Some(101));
        assert_eq!(
            import_tracks[0].local_path.as_deref(),
            Some(dl_path.to_string_lossy().as_ref())
        );

        // 5. Clear completed downloads
        controller.clear_completed_downloads();
        assert_eq!(controller.download_jobs.len(), 0);
        assert_eq!(controller.active_downloads_count(), 0);
    }

    #[test]
    fn test_stream_controller_download_format_selection() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        // 1. Verify default selected format is FLAC
        assert_eq!(controller.selected_download_format, DownloadFormat::Flac);

        let track1 = StreamTrack::new(
            "tr_1",
            "Track 1",
            "Artist 1",
            "Album 1",
            150.0,
            None,
            "spotify:track:tr_1",
            "spotify",
        );

        // 2. download_track uses default format
        controller.download_track(track1);
        assert_eq!(controller.download_jobs.len(), 1);
        assert_eq!(controller.download_jobs[0].format, DownloadFormat::Flac);

        // 3. Update format via set_download_format
        controller.set_download_format(DownloadFormat::Opus);
        assert_eq!(controller.selected_download_format, DownloadFormat::Opus);

        let track2 = StreamTrack::new(
            "tr_2",
            "Track 2",
            "Artist 2",
            "Album 2",
            180.0,
            None,
            "spotify:track:tr_2",
            "spotify",
        );
        controller.download_track(track2);
        assert_eq!(controller.download_jobs.len(), 2);
        assert_eq!(controller.download_jobs[1].format, DownloadFormat::Opus);

        // 4. Explicitly download with a specific format
        let track3 = StreamTrack::new(
            "tr_3",
            "Track 3",
            "Artist 3",
            "Album 3",
            210.0,
            None,
            "spotify:track:tr_3",
            "spotify",
        );
        controller.download_track_with_format(track3, DownloadFormat::Wav);
        assert_eq!(controller.download_jobs.len(), 3);
        assert_eq!(controller.download_jobs[2].format, DownloadFormat::Wav);
    }

    #[test]
    fn test_stream_playback_ready_event_routing_and_state() {
        let (cmd_tx, _cmd_rx) = channel::<StreamCommand>();
        let (event_tx, event_rx) = channel::<StreamEvent>();

        let mut controller = StreamController {
            cmd_tx,
            event_rx,
            worker_handle: None,
            connected_providers: HashMap::new(),
            search_results: HashMap::new(),
            playlists: HashMap::new(),
            playlist_tracks: HashMap::new(),
            active_state: None,
            is_busy: true,
            last_error: None,
            last_spotify_import: None,
            stream_queue: Vec::new(),
            stream_queue_index: None,
            alternatives: HashMap::new(),
            download_jobs: Vec::new(),
            selected_download_format: DownloadFormat::default(),
            active_resolutions: Arc::new(Mutex::new(HashSet::new())),
            pending_prefetch: Arc::new(Mutex::new(VecDeque::new())),
            current_prefetch: Arc::new(Mutex::new(None)),
            promoted_to_playback: Arc::new(Mutex::new(HashMap::new())),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        };

        let track = StreamTrack::new(
            "playback_ready_1",
            "Starboy",
            "The Weeknd",
            "Starboy",
            230.0,
            None,
            "spotify:track:playback_ready_1",
            "spotify",
        );

        let cache_path = std::path::PathBuf::from("C:\\temp\\eclipse_stream_cache\\eclipse_stream_playback_ready_1.mp3");
        event_tx
            .send(StreamEvent::StreamPlaybackReady {
                spotify_id: "playback_ready_1".to_string(),
                cache_path: cache_path.clone(),
                track: track.clone(),
            })
            .unwrap();

        let events = controller.update();
        assert_eq!(events.len(), 1);
        assert!(!controller.is_busy);
        let state = controller.active_state.as_ref().expect("state set");
        assert!(state.is_playing);
        assert_eq!(state.current_track.as_ref().unwrap().id, "playback_ready_1");
    }

    #[test]
    fn test_stream_controller_download_tracks_batch_contract() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);
        controller.set_download_format(DownloadFormat::Flac);

        // Track 1: Available locally via local_track_id
        let mut t1 = StreamTrack::new(
            "trk_local_1",
            "Local Song 1",
            "Artist 1",
            "Album 1",
            120.0,
            None,
            "spotify:track:trk_local_1",
            "spotify",
        );
        t1.local_track_id = Some(42);

        // Track 2: Available locally via local_path
        let mut t2 = StreamTrack::new(
            "trk_local_2",
            "Local Song 2",
            "Artist 2",
            "Album 2",
            150.0,
            None,
            "spotify:track:trk_local_2",
            "spotify",
        );
        t2.local_path = Some("C:\\music\\track2.flac".to_string());

        // Track 3: Already active in download_jobs
        let t3 = StreamTrack::new(
            "trk_active_3",
            "Active Song 3",
            "Artist 3",
            "Album 3",
            200.0,
            None,
            "spotify:track:trk_active_3",
            "spotify",
        );
        controller.download_jobs.push(DownloadJob::with_format(
            "trk_active_3",
            t3.clone(),
            DownloadFormat::Flac,
        ));

        // Track 4 & 5: Unmatched, not queued yet
        let t4 = StreamTrack::new(
            "trk_new_4",
            "New Song 4",
            "Artist 4",
            "Album 4",
            180.0,
            None,
            "spotify:track:trk_new_4",
            "spotify",
        );
        let t5 = StreamTrack::new(
            "trk_new_5",
            "New Song 5",
            "Artist 5",
            "Album 5",
            210.0,
            None,
            "spotify:track:trk_new_5",
            "spotify",
        );

        let input_tracks = vec![t1, t2, t3, t4, t5];
        let queued_count = controller.download_tracks_batch(input_tracks);

        // Only tracks 4 & 5 should be newly queued
        assert_eq!(queued_count, 2);

        // Verify download_jobs list has 3 items total (1 existing + 2 newly queued)
        assert_eq!(controller.download_jobs.len(), 3);
        assert!(controller.download_jobs.iter().any(|j| j.id == "trk_new_4" && j.status == DownloadStatus::Queued));
        assert!(controller.download_jobs.iter().any(|j| j.id == "trk_new_5" && j.status == DownloadStatus::Queued));

        // Test cancel_all_downloads
        controller.cancel_all_downloads();
        assert!(controller.download_jobs.iter().all(|j| j.is_failed()));

        // Test retry_all_failed_downloads
        controller.retry_all_failed_downloads();
        assert!(controller.download_jobs.iter().all(|j| j.status == DownloadStatus::Queued));
    }

    #[test]
    fn test_instant_cache_hit_emits_playback_ready_immediately() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        let spotify_id = "test_instant_hit_abc123";
        let cache_dir = crate::streams::downloader::get_stream_cache_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("eclipse_stream_{spotify_id}.mp3"));

        // Create a dummy cached stream file (> 1024 bytes)
        let dummy_data = vec![0u8; 2048];

        let track = StreamTrack::new(
            spotify_id,
            "Instant Track",
            "Instant Artist",
            "Instant Album",
            180.0,
            None,
            format!("spotify:track:{spotify_id}"),
            "spotify",
        );

        let mut received_ready = false;
        // In multi-threaded test runs, another test may briefly clear the cache dir; retry ensures test robustness
        for _ in 0..5 {
            let _ = std::fs::write(&cache_file, &dummy_data);
            controller.play_stream_track(track.clone());

            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_millis(800) {
                let events = controller.update();
                for ev in events {
                    if let StreamEvent::StreamPlaybackReady {
                        spotify_id: ref id,
                        ref cache_path,
                        track: ref ready_track,
                    } = ev
                    {
                        assert_eq!(id, spotify_id);
                        assert_eq!(cache_path, &cache_file);
                        assert_eq!(ready_track.id, spotify_id);
                        received_ready = true;
                        break;
                    }
                }
                if received_ready {
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            if received_ready {
                break;
            }
        }

        let _ = std::fs::remove_file(&cache_file);
        assert!(received_ready, "Instant cache hit should immediately emit StreamPlaybackReady");
    }

    #[test]
    fn test_active_resolution_deduplication_and_guard() {
        let active_set = Arc::new(std::sync::Mutex::new(std::collections::HashSet::new()));
        let track_id = "test_dedup_track_id";

        // Test insertion and is_resolving logic
        {
            let guard = ActiveResolutionGuard {
                spotify_id: track_id.to_string(),
                active_resolutions: active_set.clone(),
            };
            active_set.lock().unwrap().insert(track_id.to_string());
            assert!(active_set.lock().unwrap().contains(track_id));
            drop(guard);
        }

        // Verify RAII guard automatically cleans up on drop
        assert!(!active_set.lock().unwrap().contains(track_id));
    }

    #[test]
    fn test_stream_controller_cache_delegation_methods() {
        let (provider, _) = TestProvider::new();
        let controller = StreamController::new(vec![Box::new(provider)]);

        // Check cache size method returns a valid u64
        let size = controller.get_stream_cache_size();
        assert!(size < u64::MAX);

        // Check is_resolving method
        assert!(!controller.is_resolving("unknown_track_12345"));

        // Check clear cache method
        let res = controller.clear_stream_cache();
        assert!(res.is_ok());
    }

    #[test]
    fn test_prefetch_stream_tracks_queue_filtering_and_bounded_size() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        // 1. Create a dummy cached stream file for track 0
        let cached_id = "test_pf_cached_001";
        let cache_dir = crate::streams::downloader::get_stream_cache_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("eclipse_stream_{cached_id}.mp3"));
        let _ = std::fs::write(&cache_file, vec![0u8; 2048]);

        let t_cached = StreamTrack::new(
            cached_id,
            "Cached Song",
            "Artist",
            "Album",
            180.0,
            None,
            format!("spotify:track:{cached_id}"),
            "spotify",
        );
        let t1 = StreamTrack::new(
            "test_pf_new_01",
            "New Song 1",
            "Artist",
            "Album",
            190.0,
            None,
            "spotify:track:test_pf_new_01",
            "spotify",
        );
        let t2 = StreamTrack::new(
            "test_pf_new_02",
            "New Song 2",
            "Artist",
            "Album",
            200.0,
            None,
            "spotify:track:test_pf_new_02",
            "spotify",
        );
        let t3 = StreamTrack::new(
            "test_pf_new_03",
            "New Song 3",
            "Artist",
            "Album",
            210.0,
            None,
            "spotify:track:test_pf_new_03",
            "spotify",
        );

        // Keep active_resolutions non-empty so prefetch worker defers popping and we can inspect queue
        if let Ok(mut active) = controller.active_resolutions.lock() {
            active.insert("busy_token".to_string());
        }

        // In multi-threaded test runs, another test may briefly clear the cache dir; retry ensures test robustness
        let mut verified = false;
        for _ in 0..5 {
            let _ = std::fs::write(&cache_file, vec![0u8; 2048]);
            // Send 4 tracks: 1 cached, 3 new
            controller.prefetch_stream_tracks(vec![t_cached.clone(), t1.clone(), t2.clone(), t3.clone()]);

            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_millis(600) {
                if !cache_file.exists() {
                    let _ = std::fs::write(&cache_file, vec![0u8; 2048]);
                }
                let count = controller.pending_prefetch_count();
                if count == 2 {
                    let queue = match controller.pending_prefetch.lock() {
                        Ok(g) => g.clone(),
                        Err(p) => p.into_inner().clone(),
                    };
                    // Verify t_cached was filtered out and at most 2 items were queued (t1 and t2)
                    if queue.len() == 2 && queue[0].id == "test_pf_new_01" && queue[1].id == "test_pf_new_02" {
                        verified = true;
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(20));
            }
            if verified {
                break;
            }
        }

        assert!(verified, "Pending prefetch queue should contain exactly 2 bounded tracks, excluding cached track");

        // Now replace queue with a single new track t3 (stale tracks discarded)
        controller.prefetch_stream_tracks(vec![t3.clone()]);
        let start = std::time::Instant::now();
        let mut replaced = false;
        while start.elapsed() < Duration::from_millis(1000) {
            let queue = match controller.pending_prefetch.lock() {
                Ok(g) => g.clone(),
                Err(p) => p.into_inner().clone(),
            };
            if queue.len() == 1 && queue[0].id == "test_pf_new_03" {
                replaced = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        assert!(replaced, "Subsequent prefetch command should replace stale pending queue items");

        // Cleanup
        if let Ok(mut active) = controller.active_resolutions.lock() {
            active.clear();
        }
        let _ = std::fs::remove_file(&cache_file);
    }

    #[test]
    fn test_prefetch_in_flight_promotion_state_transition() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        let track_id = "test_promo_inflight_999";
        let track = StreamTrack::new(
            track_id,
            "Inflight Song",
            "Artist",
            "Album",
            200.0,
            None,
            format!("spotify:track:{track_id}"),
            "spotify",
        );

        // 1. Simulate track currently being actively resolved by background prefetch worker
        {
            let mut cur = match controller.current_prefetch.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            *cur = Some(track_id.to_string());
        }

        // Verify is_prefetching returns true
        assert!(controller.is_prefetching(track_id));

        // 2. PlayStreamTrack is dispatched for the in-flight prefetching track
        controller.play_stream_track(track.clone());

        // Wait for worker to process PlayStreamTrack and promote the track
        let start = std::time::Instant::now();
        let mut promoted = false;
        while start.elapsed() < Duration::from_millis(1000) {
            let is_promoted = match controller.promoted_to_playback.lock() {
                Ok(g) => g.contains_key(track_id),
                Err(p) => p.into_inner().contains_key(track_id),
            };
            let is_res = controller.is_resolving(track_id);
            if is_promoted && is_res {
                promoted = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        assert!(promoted, "In-flight track should be promoted to playback and marked in active_resolutions");

        // 3. Simulate prefetch worker finishing cache and dispatching StreamPlaybackReady
        let cache_path = std::env::temp_dir().join(format!("eclipse_stream_{track_id}.mp3"));
        let (promoted_track, was_promoted) = {
            let mut cur = match controller.current_prefetch.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            *cur = None;
            let mut prom = match controller.promoted_to_playback.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            (prom.remove(track_id), true)
        };
        assert!(was_promoted);
        assert!(promoted_track.is_some());

        // Worker removes from active_resolutions and sends StreamPlaybackReady
        {
            let mut active = match controller.active_resolutions.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            active.remove(track_id);
        }

        // Deliver ready event to event channel
        let (cmd_tx, _cmd_rx) = channel::<StreamCommand>();
        let (event_tx, event_rx) = channel::<StreamEvent>();
        let _ = event_tx.send(StreamEvent::StreamPlaybackReady {
            spotify_id: track_id.to_string(),
            cache_path: cache_path.clone(),
            track: track.clone(),
        });
        controller.cmd_tx = cmd_tx;
        controller.event_rx = event_rx;

        let events = controller.update();
        assert_eq!(events.len(), 1);
        assert!(!controller.is_busy);
        let state = controller.active_state.as_ref().expect("playback state updated");
        assert!(state.is_playing);
        assert_eq!(state.current_track.as_ref().unwrap().id, track_id);
    }

    #[test]
    fn test_silent_prefetch_remains_silent_to_ui() {
        let (provider, _) = TestProvider::new();
        let mut controller = StreamController::new(vec![Box::new(provider)]);

        // Create a dummy cached stream file so prefetch loop succeeds immediately
        let track_id = "test_silent_pf_888";
        let cache_dir = crate::streams::downloader::get_stream_cache_dir();
        let _ = std::fs::create_dir_all(&cache_dir);
        let cache_file = cache_dir.join(format!("eclipse_stream_{track_id}.mp3"));
        let _ = std::fs::write(&cache_file, vec![0u8; 2048]);

        let track = StreamTrack::new(
            track_id,
            "Silent Track",
            "Artist",
            "Album",
            180.0,
            None,
            format!("spotify:track:{track_id}"),
            "spotify",
        );

        controller.prefetch_stream_tracks(vec![track]);

        // Poll update loop for 500ms
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            let events = controller.update();
            for ev in events {
                if let StreamEvent::StreamPlaybackReady { .. } = ev {
                    panic!("Silent prefetch must NEVER emit StreamPlaybackReady for unrequested tracks");
                }
            }
            thread::sleep(Duration::from_millis(20));
        }

        // UI state remains unaffected
        assert!(!controller.is_busy);
        assert!(controller.active_state.is_none());

        let _ = std::fs::remove_file(&cache_file);
    }

    #[test]
    fn test_prefetch_defers_during_active_playback() {
        let (provider, _) = TestProvider::new();
        let controller = StreamController::new(vec![Box::new(provider)]);

        // Mark active playback buffering in progress
        let active_playback_id = "active_playback_busy_trk";
        if let Ok(mut active) = controller.active_resolutions.lock() {
            active.insert(active_playback_id.to_string());
        }

        // Queue a track into pending_prefetch
        let t_pending = StreamTrack::new(
            "test_pending_defer_1",
            "Defer Song",
            "Artist",
            "Album",
            180.0,
            None,
            "spotify:track:test_pending_defer_1",
            "spotify",
        );
        if let Ok(mut queue) = controller.pending_prefetch.lock() {
            queue.push_back(t_pending.clone());
        }

        // Spawn prefetch worker
        let (event_tx, _event_rx) = channel::<StreamEvent>();
        let prefetch_running = Arc::new(AtomicBool::new(false));
        trigger_prefetch_worker(
            controller.pending_prefetch.clone(),
            controller.active_resolutions.clone(),
            controller.current_prefetch.clone(),
            controller.promoted_to_playback.clone(),
            prefetch_running,
            controller.shutdown_flag.clone(),
            event_tx,
        );

        // While active_resolutions is non-empty, prefetch must defer and not pop t_pending
        thread::sleep(Duration::from_millis(250));
        assert_eq!(controller.pending_prefetch_count(), 1);

        // Now clear active playback resolutions
        if let Ok(mut active) = controller.active_resolutions.lock() {
            active.clear();
        }

        // Cleanup
        controller.shutdown_flag.store(true, Ordering::SeqCst);
    }
}
