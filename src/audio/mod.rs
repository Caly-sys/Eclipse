pub mod engine;
pub mod symphonia_decoder;
pub mod visualizer;

pub use engine::AudioEngine;
pub use visualizer::{AudioTapBuffer, Visualizer};

use crate::config::{AppConfig, AudioSettings, RepeatMode};
use crate::library::model::Track;
use crate::lyrics::{LrclibRecord, Lyrics};
use crate::streams::{PlaybackSource, StreamTrack};
use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Maximum number of entries retained in the backward navigation history stack.
const HISTORY_MAX: usize = 100;

pub struct PlayerController {
    pub engine: AudioEngine,
    pub visualizer: Visualizer,
    pub queue: Vec<Track>,
    pub current_index: Option<usize>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub playback_source: PlaybackSource,
    pub active_stream_track: Option<StreamTrack>,
    pub lyrics: Option<Lyrics>,
    pub is_loading_lyrics: bool,
    pub is_searching_lyrics: bool,
    pub manual_search_results: Option<Vec<LrclibRecord>>,
    pub manual_search_error: Option<String>,
    lyrics_rx: Receiver<Option<Lyrics>>,
    lyrics_tx: Sender<Option<Lyrics>>,
    manual_search_rx: Receiver<Result<Vec<LrclibRecord>, String>>,
    manual_search_tx: Sender<Result<Vec<LrclibRecord>, String>>,
    /// Legacy field kept for backward compatibility; now mirrors `shuffle_order`.
    shuffled_indices: Vec<usize>,
    // ── Two-Tier Queue State ──────────────────────────────────────────
    /// Tier 1: User-queued track indices (via "Add to Queue" / "Play Next").
    user_queue: VecDeque<usize>,
    /// Tier 2: Permutation shuffle order (or linear order when shuffle is off).
    shuffle_order: Vec<usize>,
    /// Position pointer inside `shuffle_order`.
    queue_pos: usize,
    /// Bounded backward navigation stack of previously played track indices.
    history: Vec<usize>,
}

impl PlayerController {
    pub fn new(cfg: &AppConfig) -> Self {
        let (tx, rx) = channel();
        let (ms_tx, ms_rx) = channel();
        let mut engine = AudioEngine::new();
        engine.set_volume(cfg.volume);
        engine.set_muted(cfg.muted);

        let visualizer = Visualizer::new(cfg.visualizer_style, cfg.visualizer_bands, engine.tap_buffer());

        let mut controller = Self {
            engine,
            visualizer,
            queue: Vec::new(),
            current_index: None,
            shuffle: cfg.shuffle,
            repeat: cfg.repeat,
            playback_source: PlaybackSource::Local,
            active_stream_track: None,
            lyrics: None,
            is_loading_lyrics: false,
            is_searching_lyrics: false,
            manual_search_results: None,
            manual_search_error: None,
            lyrics_rx: rx,
            lyrics_tx: tx,
            manual_search_rx: ms_rx,
            manual_search_tx: ms_tx,
            shuffled_indices: Vec::new(),
            user_queue: VecDeque::new(),
            shuffle_order: Vec::new(),
            queue_pos: 0,
            history: Vec::new(),
        };

        controller.apply_audio_settings(&cfg.audio);
        controller
    }

    pub fn apply_audio_settings(&mut self, settings: &AudioSettings) {
        self.engine.set_output_config(
            &settings.output_device,
            settings.buffer_ms,
            settings.exclusive_mode,
        );
        self.engine.set_fade_pause_resume_ms(settings.fade_pause_resume_ms);
        self.engine.set_crossfade_sec(settings.crossfade_sec);
        self.engine.set_gapless(settings.gapless_playback);

        let (rg_track, rg_album) = if let Some(track) = self.current_track() {
            engine::read_replaygain_from_file(Path::new(&track.path))
        } else {
            (None, None)
        };

        self.engine.set_volume_leveling(
            &settings.volume_leveling,
            settings.smart_target_lufs,
            None,
            rg_track,
            rg_album,
        );
    }

    pub fn play_pause(&mut self) {
        if self.playback_source.is_local() {
            self.engine.toggle_play_pause();
        }
    }

    pub fn toggle_playback(&mut self) {
        if self.playback_source.is_local() {
            self.engine.toggle_play_pause();
        }
    }

    pub fn is_stream_active(&self) -> bool {
        self.playback_source.is_stream()
    }

    pub fn stream_provider_id(&self) -> Option<&str> {
        self.playback_source.provider_id()
    }

    pub fn switch_to_stream(&mut self, provider_id: &str, track: Option<StreamTrack>) {
        self.engine.pause();
        self.playback_source = PlaybackSource::Stream(provider_id.to_string());
        if let Some(ref t) = track {
            self.load_lyrics_for_stream(&t.artist, &t.title, &t.album, t.duration_secs);
        }
        self.active_stream_track = track;
    }

    pub fn switch_to_local(&mut self) {
        self.playback_source = PlaybackSource::Local;
        self.active_stream_track = None;
    }

    pub fn set_active_stream_track(&mut self, track: Option<StreamTrack>) {
        if let Some(ref t) = track {
            self.load_lyrics_for_stream(&t.artist, &t.title, &t.album, t.duration_secs);
        }
        self.active_stream_track = track;
    }

    pub fn load_lyrics_for_stream(
        &mut self,
        artist: &str,
        title: &str,
        album: &str,
        duration_secs: f64,
    ) {
        self.lyrics = None;
        self.is_loading_lyrics = true;

        let tx = self.lyrics_tx.clone();
        let track_path = std::path::PathBuf::from(format!("stream_{}_{}.lrc", artist, title));
        let a = artist.to_string();
        let t = title.to_string();
        let alb = album.to_string();

        crate::lyrics::lrclib::fetch_track_lyrics_async(
            track_path,
            a,
            t,
            alb,
            duration_secs,
            move |res| {
                let _ = tx.send(res);
            },
        );
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_index.and_then(|idx| self.queue.get(idx))
    }

    pub fn load_queue(&mut self, tracks: Vec<Track>, start_index: usize) {
        self.playback_source = PlaybackSource::Local;
        self.active_stream_track = None;
        self.user_queue.clear();
        self.history.clear();
        if tracks.is_empty() {
            self.queue.clear();
            self.current_index = None;
            self.shuffled_indices.clear();
            self.shuffle_order.clear();
            self.queue_pos = 0;
            self.engine.stop();
            return;
        }

        self.queue = tracks;
        let idx = start_index.min(self.queue.len() - 1);
        self.current_index = Some(idx);
        if self.shuffle {
            self.rebuild_shuffle();
        } else {
            self.rebuild_linear_order(idx);
        }
        self.play_index(idx);
    }

    pub fn set_queue_paused(&mut self, tracks: Vec<Track>, start_index: usize) {
        self.playback_source = PlaybackSource::Local;
        self.active_stream_track = None;
        self.user_queue.clear();
        self.history.clear();
        if tracks.is_empty() {
            return;
        }
        self.queue = tracks;
        let idx = start_index.min(self.queue.len() - 1);
        self.current_index = Some(idx);
        if self.shuffle {
            self.rebuild_shuffle();
        } else {
            self.rebuild_linear_order(idx);
        }
        let track = self.queue[idx].clone();
        let path = Path::new(&track.path);
        let _ = self.engine.play_file(path, track.duration_secs);
        self.engine.pause();
        self.load_lyrics_for_track(&track);
    }

    pub fn play_index(&mut self, index: usize) {
        self.play_index_internal(index, true);
    }

    fn play_index_internal(&mut self, index: usize, sync_queue_pos: bool) {
        if index >= self.queue.len() {
            return;
        }

        self.playback_source = PlaybackSource::Local;
        self.active_stream_track = None;
        self.current_index = Some(index);

        // Sync queue_pos to current position in shuffle_order / linear order if requested
        if sync_queue_pos {
            if let Some(pos) = self.shuffle_order.iter().position(|&x| x == index) {
                self.queue_pos = pos;
            }
        }

        let track = self.queue[index].clone();
        let path = Path::new(&track.path);

        let (rg_track, rg_album) = engine::read_replaygain_from_file(path);
        let mode = self.engine.volume_leveling_mode().to_string();
        let target_lufs = self.engine.smart_target_lufs();
        self.engine.set_volume_leveling(
            &mode,
            target_lufs,
            None,
            rg_track,
            rg_album,
        );

        if let Err(e) = self.engine.play_file(path, track.duration_secs) {
            eprintln!("Playback error: {}", e);
        }

        self.load_lyrics_for_track(&track);
    }

    pub fn load_lyrics_for_track(&mut self, track: &Track) {
        self.lyrics = None;
        self.is_loading_lyrics = true;

        let track_path = Path::new(&track.path).to_path_buf();

        // Step A: Local first (sidecar, cache, or embedded)
        if let Some(local) = Lyrics::load_with_cache(&track_path) {
            self.lyrics = Some(local);
            self.is_loading_lyrics = false;
            return;
        }

        // Steps B & C: Fetch online with fallback and cache
        let tx = self.lyrics_tx.clone();
        let artist = track.artist.clone();
        let title = track.title.clone();
        let album = track.album.clone();
        let duration = track.duration_secs;

        crate::lyrics::lrclib::fetch_track_lyrics_async(
            track_path,
            artist,
            title,
            album,
            duration,
            move |res| {
                let _ = tx.send(res);
            },
        );
    }

    pub fn search_lyrics_manual(
        &mut self,
        query: &str,
        track_name: Option<&str>,
        artist_name: Option<&str>,
    ) {
        self.is_searching_lyrics = true;
        self.manual_search_results = None;
        self.manual_search_error = None;

        let tx = self.manual_search_tx.clone();
        let q = query.to_string();
        let t = track_name.map(|s| s.to_string());
        let a = artist_name.map(|s| s.to_string());

        thread::spawn(move || {
            let res = crate::lyrics::search_lrclib_manual(&q, t.as_deref(), a.as_deref());
            let _ = tx.send(res);
        });
    }

    pub fn apply_custom_lyrics(&mut self, record: &LrclibRecord) {
        if let Some(track) = self.current_track() {
            let track_path = Path::new(&track.path);
            let lyrics = crate::lyrics::lrclib::apply_and_cache_record(track_path, record);
            self.lyrics = Some(lyrics);
        } else {
            self.lyrics = Some(record.to_lyrics());
        }
    }

    pub fn clear_manual_search(&mut self) {
        self.is_searching_lyrics = false;
        self.manual_search_results = None;
        self.manual_search_error = None;
    }

    pub fn next(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        // Push outgoing track to history
        if let Some(curr) = self.current_index {
            self.history.push(curr);
            if self.history.len() > HISTORY_MAX {
                self.history.remove(0);
            }
        }

        // Priority 1: User queue (Tier 1 — "Next in Queue")
        if let Some(queued_idx) = self.user_queue.pop_front() {
            if queued_idx < self.queue.len() {
                self.play_index_internal(queued_idx, false);
                return;
            }
            // Invalid index was in user_queue, skip it and fall through
        }

        // Priority 2: Playlist queue (Tier 2 — shuffle_order / linear)
        if self.shuffle_order.is_empty() {
            if self.shuffle {
                self.rebuild_shuffle();
            } else if let Some(curr) = self.current_index {
                self.rebuild_linear_order(curr);
            } else {
                self.rebuild_linear_order(0);
            }
        }

        if self.queue_pos + 1 < self.shuffle_order.len() {
            self.queue_pos += 1;
            let next_idx = self.shuffle_order[self.queue_pos];
            self.play_index(next_idx);
        } else if self.repeat == RepeatMode::All {
            if self.shuffle {
                let last_idx = self.current_index;
                self.rebuild_shuffle();
                // Ensure the last song doesn't immediately repeat
                if self.queue.len() > 1 {
                    if let Some(last) = last_idx {
                        if self.shuffle_order.first().copied() == Some(last)
                            && self.shuffle_order.len() > 1
                        {
                            self.shuffle_order.swap(0, 1);
                        }
                    }
                }
            }
            self.queue_pos = 0;
            if let Some(&first) = self.shuffle_order.first() {
                self.play_index(first);
            }
        } else {
            self.engine.stop();
        }
    }

    pub fn prev(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        // If played more than 3 seconds, restart current track
        if self.engine.position().as_secs() >= 3 {
            self.engine.seek(Duration::ZERO);
            return;
        }

        // Priority 1: Pop from backward navigation history
        if let Some(prev_idx) = self.history.pop() {
            if prev_idx < self.queue.len() {
                // Sync queue_pos to the history target
                if let Some(pos) = self.shuffle_order.iter().position(|&x| x == prev_idx) {
                    self.queue_pos = pos;
                }
                self.play_index(prev_idx);
                return;
            }
        }

        // Priority 2: Step backward in shuffle_order / linear order
        if self.shuffle_order.is_empty() {
            if self.shuffle {
                self.rebuild_shuffle();
            } else if let Some(curr) = self.current_index {
                self.rebuild_linear_order(curr);
            } else {
                self.rebuild_linear_order(0);
            }
        }

        if self.queue_pos > 0 {
            self.queue_pos -= 1;
            let prev_idx = self.shuffle_order[self.queue_pos];
            self.play_index(prev_idx);
        } else if self.repeat == RepeatMode::All && !self.shuffle_order.is_empty() {
            self.queue_pos = self.shuffle_order.len() - 1;
            let last_idx = self.shuffle_order[self.queue_pos];
            self.play_index(last_idx);
        } else {
            self.engine.seek(Duration::ZERO);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            self.rebuild_shuffle();
        } else {
            // Seamless linear resume: sync queue_pos to current_index
            if let Some(curr) = self.current_index {
                self.rebuild_linear_order(curr);
            } else {
                self.shuffle_order = (0..self.queue.len()).collect();
                self.shuffled_indices = self.shuffle_order.clone();
                self.queue_pos = 0;
            }
        }
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = match self.repeat {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
    }

    fn rebuild_shuffle(&mut self) {
        let count = self.queue.len();
        if count == 0 {
            self.shuffled_indices.clear();
            self.shuffle_order.clear();
            self.queue_pos = 0;
            return;
        }
        let mut indices: Vec<usize> = (0..count).collect();
        let mut rng = rand::thread_rng();
        indices.shuffle(&mut rng);

        // Pin the currently playing track at position 0
        if let Some(curr) = self.current_index {
            if let Some(pos) = indices.iter().position(|&x| x == curr) {
                indices.remove(pos);
                indices.insert(0, curr);
            }
        }

        // Smart Artist Spread: avoid back-to-back tracks by the same artist
        if indices.len() > 2 {
            for i in 1..indices.len() - 1 {
                let artist_i = self.queue.get(indices[i]).map(|t| t.artist.as_str()).unwrap_or("");
                if artist_i.is_empty() {
                    continue;
                }
                let artist_prev = self.queue.get(indices[i - 1]).map(|t| t.artist.as_str()).unwrap_or("");
                if artist_i == artist_prev {
                    // Find the next index with a different artist to swap with
                    let swap_target = indices.iter().enumerate().skip(i + 1).find_map(|(j, &idx)| {
                        let artist_j = self.queue.get(idx).map(|t| t.artist.as_str()).unwrap_or("");
                        if artist_j != artist_i {
                            Some(j)
                        } else {
                            None
                        }
                    });
                    if let Some(j) = swap_target {
                        indices.swap(i, j);
                    }
                }
            }
        }

        self.shuffle_order = indices.clone();
        self.shuffled_indices = indices;
        self.queue_pos = 0;
    }

    /// Build a linear (non-shuffled) play order starting from the given index.
    fn rebuild_linear_order(&mut self, start_idx: usize) {
        let count = self.queue.len();
        if count == 0 {
            self.shuffle_order.clear();
            self.shuffled_indices.clear();
            self.queue_pos = 0;
            return;
        }
        self.shuffle_order = (0..count).collect();
        self.shuffled_indices = self.shuffle_order.clone();
        self.queue_pos = start_idx.min(count.saturating_sub(1));
    }

    pub fn peek_next_index(&self) -> Option<usize> {
        if self.queue.is_empty() {
            return None;
        }

        // Priority 1: User queue
        if let Some(&queued_idx) = self.user_queue.front() {
            if queued_idx < self.queue.len() {
                return Some(queued_idx);
            }
        }

        // Priority 2: Playlist queue (shuffle_order / linear)
        if self.shuffle_order.is_empty() {
            return None;
        }

        if self.queue_pos + 1 < self.shuffle_order.len() {
            return Some(self.shuffle_order[self.queue_pos + 1]);
        } else if self.repeat == RepeatMode::All {
            return self.shuffle_order.first().copied();
        }

        None
    }

    pub fn update(&mut self) {
        // Poll background lyrics
        if let Ok(res) = self.lyrics_rx.try_recv() {
            self.lyrics = res;
            self.is_loading_lyrics = false;
        }

        // Poll manual lyrics search
        if let Ok(res) = self.manual_search_rx.try_recv() {
            self.is_searching_lyrics = false;
            match res {
                Ok(records) => {
                    self.manual_search_results = Some(records);
                    self.manual_search_error = None;
                }
                Err(err) => {
                    self.manual_search_results = None;
                    self.manual_search_error = Some(err);
                }
            }
        }

        // Advance crossfade transition if active
        self.engine.update_crossfade();

        // Gapless pre-queueing when approaching EOF
        if self.engine.is_gapless()
            && self.engine.crossfade_sec() == 0.0
            && !self.engine.has_prequeued()
            && self.engine.is_approaching_eof(Duration::from_millis(1500))
        {
            if let Some(next_idx) = self.peek_next_index() {
                if let Some(next_track) = self.queue.get(next_idx) {
                    let _ = self.engine.prequeue_file(Path::new(&next_track.path));
                }
            }
        }

        let is_playing = self.engine.is_playing();
        let pos_secs = self.engine.position().as_secs_f64();
        self.visualizer.update(is_playing, pos_secs);

        // Gapless seamless transition check
        if self.engine.has_prequeued() && self.engine.has_track_reached_end() {
            if let Some(next_idx) = self.peek_next_index() {
                if let Some(next_track) = self.queue.get(next_idx).cloned() {
                    let next_path = Path::new(&next_track.path);
                    self.engine.transition_to_prequeued(next_path, next_track.duration_secs);
                    // Push outgoing track to history
                    if let Some(curr) = self.current_index {
                        self.history.push(curr);
                        if self.history.len() > HISTORY_MAX {
                            self.history.remove(0);
                        }
                    }
                    // Advance queue state (user_queue or queue_pos)
                    let from_user_queue = !self.user_queue.is_empty();
                    if from_user_queue {
                        self.user_queue.pop_front();
                    } else if self.queue_pos + 1 < self.shuffle_order.len() {
                        self.queue_pos += 1;
                    }
                    self.current_index = Some(next_idx);
                    if !from_user_queue {
                        if let Some(pos) = self.shuffle_order.iter().position(|&x| x == next_idx) {
                            self.queue_pos = pos;
                        }
                    }
                    self.load_lyrics_for_track(&next_track);
                    return;
                }
            }
        }

        // Check if track ended (EOF)
        if self.current_index.is_some() && self.engine.is_finished() && !self.engine.is_paused() {
            if self.repeat == RepeatMode::One {
                if let Some(curr) = self.current_index {
                    self.play_index(curr);
                }
            } else {
                self.next();
            }
        }
    }

    // ── Additive Queue Operations ────────────────────────────────────

    /// Push a track index to the back of the user queue (Tier 1).
    pub fn add_to_queue(&mut self, track_index: usize) {
        if track_index < self.queue.len() {
            self.user_queue.push_back(track_index);
        }
    }

    /// Push a track index to the front of the user queue ("Play Next").
    pub fn play_next(&mut self, track_index: usize) {
        if track_index < self.queue.len() {
            self.user_queue.push_front(track_index);
        }
    }

    /// Remove a single item from the user queue by its position in `user_queue`.
    pub fn remove_from_user_queue(&mut self, queue_idx: usize) -> Option<usize> {
        if queue_idx < self.user_queue.len() {
            self.user_queue.remove(queue_idx)
        } else {
            None
        }
    }

    /// Clear the entire user queue.
    pub fn clear_user_queue(&mut self) {
        self.user_queue.clear();
    }

    /// Returns a snapshot of the current queue state for UI display.
    pub fn get_queue_view(&self) -> QueueView<'_> {
        let now_playing = self.current_index.and_then(|idx| self.queue.get(idx));

        let next_in_queue: Vec<&Track> = self
            .user_queue
            .iter()
            .filter_map(|&idx| self.queue.get(idx))
            .collect();

        let next_from_playlist: Vec<&Track> = if self.shuffle_order.is_empty() {
            Vec::new()
        } else {
            let start = (self.queue_pos + 1).min(self.shuffle_order.len());
            self.shuffle_order[start..]
                .iter()
                .filter_map(|&idx| self.queue.get(idx))
                .collect()
        };

        QueueView {
            now_playing,
            next_in_queue,
            next_from_playlist,
        }
    }

    /// Returns a reference to the user queue for UI inspection.
    pub fn user_queue_indices(&self) -> &VecDeque<usize> {
        &self.user_queue
    }

    /// Returns the backward navigation history for UI inspection.
    pub fn playback_history(&self) -> &[usize] {
        &self.history
    }
}

/// Snapshot of the current playback queue state for UI display.
pub struct QueueView<'a> {
    pub now_playing: Option<&'a Track>,
    pub next_in_queue: Vec<&'a Track>,
    pub next_from_playlist: Vec<&'a Track>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_track(id: i64, title: &str) -> Track {
        Track {
            id,
            path: format!("C:/music/{}.flac", title),
            title: title.to_string(),
            artist: "Test Artist".to_string(),
            album: "Test Album".to_string(),
            album_artist: "Test Artist".to_string(),
            genre: "Ambient".to_string(),
            year: 2024,
            track_number: id as u32,
            duration_secs: 180.0,
            bitrate_kbps: 1411,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 10_000_000,
            play_count: 0,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    #[test]
    fn test_load_queue_and_progression() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::Off;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
        ];

        player.load_queue(tracks, 0);
        assert_eq!(player.queue.len(), 3);
        assert_eq!(player.current_index, Some(0));
        assert_eq!(player.current_track().unwrap().title, "Track 1");

        // Next advances to Track 2
        player.next();
        assert_eq!(player.current_index, Some(1));
        assert_eq!(player.current_track().unwrap().title, "Track 2");

        // Next advances to Track 3
        player.next();
        assert_eq!(player.current_index, Some(2));
        assert_eq!(player.current_track().unwrap().title, "Track 3");

        // Next at end with Repeat Off stops
        player.next();
        assert!(!player.engine.is_playing());

        // Previous from Track 3 goes back to Track 2 (via history)
        player.current_index = Some(2);
        player.queue_pos = 2;
        // Seed history so prev() can pop from it
        player.history.push(1);
        player.prev();
        assert_eq!(player.current_index, Some(1));
        assert_eq!(player.current_track().unwrap().title, "Track 2");
    }

    #[test]
    fn test_repeat_all_loops_queue() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::All;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
        ];

        player.load_queue(tracks, 1);
        assert_eq!(player.current_index, Some(1));

        // Next at end wraps to 0 (Repeat All)
        player.next();
        assert_eq!(player.current_index, Some(0));

        // Prev at 0 goes back to 1 via history (next() pushed 1 to history)
        player.prev();
        assert_eq!(player.current_index, Some(1));
    }

    #[test]
    fn test_shuffle_toggle_and_progression() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let tracks = vec![
            dummy_track(1, "A"),
            dummy_track(2, "B"),
            dummy_track(3, "C"),
            dummy_track(4, "D"),
        ];

        player.load_queue(tracks, 0);
        player.toggle_shuffle();
        assert!(player.shuffle);
        assert_eq!(player.shuffled_indices.len(), 4);
        assert_eq!(player.shuffled_indices[0], 0); // Active track kept at start of shuffle

        player.next();
        assert!(player.current_index.is_some());
        assert_ne!(player.current_index, Some(0));
    }

    #[test]
    fn test_cycle_repeat_modes() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.repeat = RepeatMode::Off;

        player.cycle_repeat();
        assert_eq!(player.repeat, RepeatMode::All);

        player.cycle_repeat();
        assert_eq!(player.repeat, RepeatMode::One);

        player.cycle_repeat();
        assert_eq!(player.repeat, RepeatMode::Off);
    }

    #[test]
    fn test_apply_custom_lyrics() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let tracks = vec![dummy_track(1, "Test Song")];
        player.load_queue(tracks, 0);

        let record = LrclibRecord {
            id: 999,
            name: None,
            track_name: "Test Song".to_string(),
            artist_name: "Test Artist".to_string(),
            album_name: None,
            duration: Some(180.0),
            instrumental: false,
            plain_lyrics: None,
            synced_lyrics: Some("[00:15.00] Custom lyrics test".to_string()),
        };

        player.apply_custom_lyrics(&record);
        assert!(player.lyrics.is_some());
        let lyrics = player.lyrics.as_ref().unwrap();
        assert!(lyrics.is_synced);
        assert_eq!(lyrics.lines[0].text, "Custom lyrics test");
        assert_eq!(lyrics.lines[0].timestamp_ms, 15000);
    }

    #[test]
    fn test_apply_audio_settings() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let settings = AudioSettings {
            output_device: "System default".to_string(),
            buffer_ms: 240,
            exclusive_mode: true,
            fade_pause_resume_ms: 300,
            crossfade_sec: 4.0,
            gapless_playback: false,
            volume_leveling: "smart_lufs".to_string(),
            smart_target_lufs: -16.0,
        };

        player.apply_audio_settings(&settings);

        assert_eq!(player.engine.buffer_ms(), 240);
        assert!(player.engine.exclusive_mode());
        assert_eq!(player.engine.fade_pause_resume_ms(), 300);
        assert_eq!(player.engine.crossfade_sec(), 4.0);
        assert!(!player.engine.is_gapless());
        assert_eq!(player.engine.volume_leveling_mode(), "smart_lufs");
        assert_eq!(player.engine.smart_target_lufs(), -16.0);
    }

    #[test]
    fn test_player_play_pause_and_toggle_playback() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        assert!(!player.engine.is_paused());
        player.toggle_playback();
        player.play_pause();
    }

    #[test]
    fn test_peek_next_index_progression() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::Off;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
        ];
        player.load_queue(tracks, 0);

        assert_eq!(player.peek_next_index(), Some(1));

        player.current_index = Some(1);
        player.queue_pos = 1;
        assert_eq!(player.peek_next_index(), Some(2));

        player.current_index = Some(2);
        player.queue_pos = 2;
        assert_eq!(player.peek_next_index(), None);

        player.repeat = RepeatMode::All;
        assert_eq!(player.peek_next_index(), Some(0));
    }

    #[test]
    fn test_player_controller_stream_routing_and_switching() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        // Initially local
        assert!(player.playback_source.is_local());
        assert!(!player.is_stream_active());
        assert_eq!(player.stream_provider_id(), None);
        assert!(player.active_stream_track.is_none());

        // Switch to stream
        let stream_track = StreamTrack::new(
            "stream_1",
            "Midnight City",
            "M83",
            "Hurry Up, We're Dreaming",
            243.0,
            None,
            "spotify:track:stream_1",
            "spotify",
        );
        player.switch_to_stream("spotify", Some(stream_track.clone()));
        assert!(!player.playback_source.is_local());
        assert!(player.is_stream_active());
        assert_eq!(player.stream_provider_id(), Some("spotify"));
        assert_eq!(player.active_stream_track.as_ref().unwrap().title, "Midnight City");

        // Switching back to local via switch_to_local
        player.switch_to_local();
        assert!(player.playback_source.is_local());
        assert!(!player.is_stream_active());
        assert!(player.active_stream_track.is_none());

        // Switch to stream again, then load_queue
        player.switch_to_stream("youtube", None);
        assert!(player.is_stream_active());

        let tracks = vec![dummy_track(1, "Local Track 1")];
        player.load_queue(tracks, 0);
        // Loading local queue automatically resets playback source to Local
        assert!(player.playback_source.is_local());
        assert!(!player.is_stream_active());
        assert!(player.active_stream_track.is_none());
    }

    fn dummy_track_with_artist(id: i64, title: &str, artist: &str) -> Track {
        let mut t = dummy_track(id, title);
        t.artist = artist.to_string();
        t
    }

    #[test]
    fn test_user_queue_add_and_play_next() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::Off;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
            dummy_track(4, "Track 4"),
        ];
        player.load_queue(tracks, 0);

        // Add Track 4 to queue (back of user queue)
        player.add_to_queue(3);
        // Play Next Track 3 (front of user queue)
        player.play_next(2);

        assert_eq!(player.user_queue_indices().len(), 2);
        assert_eq!(player.user_queue_indices()[0], 2);
        assert_eq!(player.user_queue_indices()[1], 3);

        // First next() should drain user queue front: Track 3 (index 2)
        player.next();
        assert_eq!(player.current_index, Some(2));
        assert_eq!(player.current_track().unwrap().title, "Track 3");
        assert_eq!(player.user_queue_indices().len(), 1);

        // Second next() should drain user queue: Track 4 (index 3)
        player.next();
        assert_eq!(player.current_index, Some(3));
        assert_eq!(player.current_track().unwrap().title, "Track 4");
        assert_eq!(player.user_queue_indices().len(), 0);

        // Third next() should resume playlist progression from queue_pos (which was 0 -> now 1)
        player.next();
        assert_eq!(player.current_index, Some(1));
        assert_eq!(player.current_track().unwrap().title, "Track 2");
    }

    #[test]
    fn test_user_queue_does_not_advance_queue_pos() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::Off;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
        ];
        player.load_queue(tracks, 0);
        assert_eq!(player.queue_pos, 0);

        player.add_to_queue(2);
        player.next(); // Plays index 2 from user queue
        assert_eq!(player.current_index, Some(2));
        // queue_pos should still be 0 because it was a user queue item
        assert_eq!(player.queue_pos, 0);

        // Next item should be from shuffle_order / linear at queue_pos + 1 = 1
        player.next();
        assert_eq!(player.current_index, Some(1));
        assert_eq!(player.queue_pos, 1);
    }

    #[test]
    fn test_history_stack_and_prev() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::Off;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
        ];
        player.load_queue(tracks, 0);

        // Advance to 1, then to 2
        player.next();
        assert_eq!(player.playback_history(), &[0]);
        player.next();
        assert_eq!(player.playback_history(), &[0, 1]);

        // Prev should pop 1 from history
        player.prev();
        assert_eq!(player.current_index, Some(1));
        assert_eq!(player.playback_history(), &[0]);

        // Prev should pop 0 from history
        player.prev();
        assert_eq!(player.current_index, Some(0));
        assert!(player.playback_history().is_empty());
    }

    #[test]
    fn test_history_bounded_at_100() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;
        player.repeat = RepeatMode::All;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
        ];
        player.load_queue(tracks, 0);

        for _ in 0..150 {
            player.next();
        }

        assert_eq!(player.playback_history().len(), 100);
    }

    #[test]
    fn test_shuffle_artist_spread() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let tracks = vec![
            dummy_track_with_artist(1, "Song 1", "Artist A"),
            dummy_track_with_artist(2, "Song 2", "Artist A"),
            dummy_track_with_artist(3, "Song 3", "Artist B"),
            dummy_track_with_artist(4, "Song 4", "Artist A"),
            dummy_track_with_artist(5, "Song 5", "Artist C"),
        ];
        player.load_queue(tracks, 0);
        player.toggle_shuffle();

        assert!(player.shuffle);
        assert_eq!(player.shuffle_order.len(), 5);
        // Active track pinned at index 0
        assert_eq!(player.shuffle_order[0], 0);

        // Verify that not all Artist A tracks ended up clustered adjacent
        let mut adjacent_same_artist_count = 0;
        for i in 0..player.shuffle_order.len() - 1 {
            let a1 = &player.queue[player.shuffle_order[i]].artist;
            let a2 = &player.queue[player.shuffle_order[i + 1]].artist;
            if a1 == a2 {
                adjacent_same_artist_count += 1;
            }
        }
        // With 3 Artist A, 1 B, 1 C, smart spread should prevent clustering
        assert!(adjacent_same_artist_count <= 1);
    }

    #[test]
    fn test_shuffle_off_resumes_linear() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
            dummy_track(4, "Track 4"),
        ];
        player.load_queue(tracks, 0);
        player.toggle_shuffle();
        assert!(player.shuffle);

        // Force current to track 2
        player.play_index(2);
        assert_eq!(player.current_index, Some(2));

        // Toggle shuffle off
        player.toggle_shuffle();
        assert!(!player.shuffle);

        // Linear order should be 0..4, queue_pos should be 2
        assert_eq!(player.shuffle_order, vec![0, 1, 2, 3]);
        assert_eq!(player.queue_pos, 2);

        // Next should play index 3
        player.next();
        assert_eq!(player.current_index, Some(3));
    }

    #[test]
    fn test_play_index_syncs_queue_pos() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
        ];
        player.load_queue(tracks, 0);

        player.play_index(2);
        assert_eq!(player.current_index, Some(2));
        assert_eq!(player.queue_pos, 2);
    }

    #[test]
    fn test_get_queue_view() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        player.shuffle = false;

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
            dummy_track(4, "Track 4"),
        ];
        player.load_queue(tracks, 0);

        player.add_to_queue(3);

        let view = player.get_queue_view();
        assert_eq!(view.now_playing.map(|t| t.title.as_str()), Some("Track 1"));
        assert_eq!(view.next_in_queue.len(), 1);
        assert_eq!(view.next_in_queue[0].title, "Track 4");
        // next_from_playlist should contain tracks 2 and 3 (indices 1 and 2, plus 3)
        assert_eq!(view.next_from_playlist.len(), 3);
        assert_eq!(view.next_from_playlist[0].title, "Track 2");
    }

    #[test]
    fn test_clear_and_remove_user_queue() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);

        let tracks = vec![
            dummy_track(1, "Track 1"),
            dummy_track(2, "Track 2"),
            dummy_track(3, "Track 3"),
        ];
        player.load_queue(tracks, 0);

        player.add_to_queue(1);
        player.add_to_queue(2);
        assert_eq!(player.user_queue_indices().len(), 2);

        // Remove item at index 0
        let removed = player.remove_from_user_queue(0);
        assert_eq!(removed, Some(1));
        assert_eq!(player.user_queue_indices().len(), 1);
        assert_eq!(player.user_queue_indices()[0], 2);

        // Out of bounds remove
        assert_eq!(player.remove_from_user_queue(5), None);

        // Clear
        player.clear_user_queue();
        assert!(player.user_queue_indices().is_empty());
    }
}
