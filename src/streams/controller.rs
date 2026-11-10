use crate::streams::credentials::CredentialStore;
use crate::streams::playback::{
    prefetch_next_track_match, resolve_or_cached_match, search_youtube_alternatives,
    select_alternative_match, YouTubeAlternativeMatch, YouTubeWebviewPlayer,
};
use crate::streams::provider::{
    StreamPlaybackState, StreamPlaylist, StreamProvider, StreamTrack,
};
use crate::streams::spotify::SpotifyProvider;
use crate::streams::spotify_public::{
    PublicSpotifyFetcher, SpotifyImportResult, SpotifyPublicMetadataFetcher,
};
use crate::streams::youtube::YouTubeProvider;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
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

        let worker_handle = thread::Builder::new()
            .name("eclipse-stream-worker".to_string())
            .spawn(move || {
                run_stream_worker(providers, cmd_rx, event_tx);
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
        if provider_id.eq_ignore_ascii_case("youtube") && self.has_next_in_queue() {
            self.play_next_in_queue();
        } else {
            let _ = self.send(StreamCommand::Next {
                provider_id: provider_id.to_string(),
            });
        }
    }

    pub fn prev(&mut self, provider_id: &str) {
        if provider_id.eq_ignore_ascii_case("youtube")
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

    /// Load and play a stream queue.
    pub fn load_stream_queue(&mut self, tracks: Vec<StreamTrack>, start_index: usize) {
        self.stream_queue = tracks;
        self.stream_queue_index = Some(start_index);
        if let Some(track) = self.stream_queue.get(start_index).cloned() {
            self.play_stream_track(track);
            if let Some(next) = self.stream_queue.get(start_index + 1).cloned() {
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
                if let Some(peek) = self.stream_queue.get(next_idx + 1).cloned() {
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
                StreamEvent::LibraryMatched(_) => {
                    self.is_busy = false;
                }
                StreamEvent::PlaylistSaved { .. } => {
                    self.is_busy = false;
                }
                StreamEvent::StreamTrackEnded => {
                    if let Some(ref mut state) = self.active_state {
                        state.is_playing = false;
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
        let _ = self.cmd_tx.send(StreamCommand::Shutdown);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Worker thread loop: executes network I/O and provider requests off the UI thread.
fn run_stream_worker(
    mut providers: Vec<Box<dyn StreamProvider>>,
    cmd_rx: Receiver<StreamCommand>,
    event_tx: Sender<StreamEvent>,
) {
    let mut provider_map: HashMap<String, usize> = HashMap::new();
    for (idx, p) in providers.iter().enumerate() {
        provider_map.insert(p.id().to_lowercase(), idx);
    }

    let mut active_provider_id: Option<String> = None;
    let mut webview_player: Option<YouTubeWebviewPlayer> = None;
    let poll_interval = std::time::Duration::from_millis(200);

    loop {
        let cmd_opt = match cmd_rx.recv_timeout(poll_interval) {
            Ok(cmd) => Some(cmd),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => None,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        };

        if let Some(cmd) = cmd_opt {
            match cmd {
                StreamCommand::Shutdown => {
                    if let Some(mut p) = webview_player {
                        p.shutdown();
                    }
                    break;
                }
                StreamCommand::PlayStreamTrack(track) => {
                    active_provider_id = Some("youtube".to_string());
                    let tx = event_tx.clone();
                    let track_clone = track.clone();
                    match crate::library::Database::open() {
                        Ok(db) => {
                            let api_key = CredentialStore::get_youtube_api_key().ok().flatten();
                            match resolve_or_cached_match(&db, api_key.as_deref(), &track) {
                                Ok(matched) => {
                                    let player = webview_player.get_or_insert_with(YouTubeWebviewPlayer::new);
                                    let _ = player.play_video(&matched.video_id, &track_clone);
                                    let _ = tx.send(StreamEvent::TrackStarted {
                                        provider_id: "youtube".to_string(),
                                        track: track_clone,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(StreamEvent::Error {
                                        provider_id: "youtube".to_string(),
                                        action: "play_stream_track".to_string(),
                                        message: e,
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(StreamEvent::Error {
                                provider_id: "youtube".to_string(),
                                action: "play_stream_track".to_string(),
                                message: format!("Failed to open database: {e}"),
                            });
                        }
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
                    if id_key == "youtube" {
                        if let Some(ref p) = webview_player {
                            p.pause();
                        }
                    }
                    if let Some(&idx) = provider_map.get(&id_key) {
                        let _ = providers[idx].pause();
                    }
                }
                StreamCommand::Resume { provider_id } => {
                    let id_key = provider_id.to_lowercase();
                    active_provider_id = Some(provider_id.clone());
                    if id_key == "youtube" {
                        if let Some(ref p) = webview_player {
                            p.resume();
                        }
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
                    if id_key == "youtube" {
                        if let Some(ref p) = webview_player {
                            p.seek(position_secs);
                        }
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
                    if id_key == "youtube" {
                        if let Some(ref p) = webview_player {
                            p.set_volume(volume);
                        }
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
                        let fetcher = PublicSpotifyFetcher::new();
                        let res = fetcher.fetch_metadata(&url);
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
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(50));
            for ev in controller.update() {
                if let StreamEvent::PlaylistSaved { name, track_count, .. } = ev {
                    assert_eq!(name, "My Test Pl");
                    assert_eq!(track_count, 1);
                    got_saved = true;
                    break;
                }
            }
            if got_saved {
                break;
            }
        }
        assert!(got_saved);
        assert!(!controller.is_busy);
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
}
