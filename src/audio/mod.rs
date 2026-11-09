pub mod engine;
pub mod visualizer;

pub use engine::AudioEngine;
pub use visualizer::{AudioTapBuffer, Visualizer};

use crate::config::{AppConfig, AudioSettings, RepeatMode};
use crate::library::model::Track;
use crate::lyrics::{LrclibRecord, Lyrics};
use rand::seq::SliceRandom;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct PlayerController {
    pub engine: AudioEngine,
    pub visualizer: Visualizer,
    pub queue: Vec<Track>,
    pub current_index: Option<usize>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub lyrics: Option<Lyrics>,
    pub is_loading_lyrics: bool,
    pub is_searching_lyrics: bool,
    pub manual_search_results: Option<Vec<LrclibRecord>>,
    pub manual_search_error: Option<String>,
    lyrics_rx: Receiver<Option<Lyrics>>,
    lyrics_tx: Sender<Option<Lyrics>>,
    manual_search_rx: Receiver<Result<Vec<LrclibRecord>, String>>,
    manual_search_tx: Sender<Result<Vec<LrclibRecord>, String>>,
    shuffled_indices: Vec<usize>,
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
        self.engine.toggle_play_pause();
    }

    pub fn toggle_playback(&mut self) {
        self.engine.toggle_play_pause();
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_index.and_then(|idx| self.queue.get(idx))
    }

    pub fn load_queue(&mut self, tracks: Vec<Track>, start_index: usize) {
        if tracks.is_empty() {
            self.queue.clear();
            self.current_index = None;
            self.shuffled_indices.clear();
            self.engine.stop();
            return;
        }

        self.queue = tracks;
        let idx = start_index.min(self.queue.len() - 1);
        self.current_index = Some(idx);
        if self.shuffle {
            self.rebuild_shuffle();
        }
        self.play_index(idx);
    }

    pub fn play_index(&mut self, index: usize) {
        if index >= self.queue.len() {
            return;
        }

        self.current_index = Some(index);
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

        if self.shuffle {
            if self.shuffled_indices.is_empty() {
                self.rebuild_shuffle();
            }

            if let Some(curr) = self.current_index {
                if let Some(pos) = self.shuffled_indices.iter().position(|&x| x == curr) {
                    if pos + 1 < self.shuffled_indices.len() {
                        let next_idx = self.shuffled_indices[pos + 1];
                        self.play_index(next_idx);
                        return;
                    } else if self.repeat == RepeatMode::All {
                        self.rebuild_shuffle();
                        if let Some(&first) = self.shuffled_indices.first() {
                            self.play_index(first);
                            return;
                        }
                    } else {
                        self.engine.stop();
                        return;
                    }
                }
            } else if let Some(&first) = self.shuffled_indices.first() {
                self.play_index(first);
                return;
            }
        }

        // Sequential (non-shuffle) playback
        if let Some(curr) = self.current_index {
            if curr + 1 < self.queue.len() {
                self.play_index(curr + 1);
            } else if self.repeat == RepeatMode::All {
                self.play_index(0);
            } else {
                self.engine.stop();
            }
        } else if !self.queue.is_empty() {
            self.play_index(0);
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

        if self.shuffle {
            if self.shuffled_indices.is_empty() {
                self.rebuild_shuffle();
            }

            if let Some(curr) = self.current_index {
                if let Some(pos) = self.shuffled_indices.iter().position(|&x| x == curr) {
                    if pos > 0 {
                        let prev_idx = self.shuffled_indices[pos - 1];
                        self.play_index(prev_idx);
                        return;
                    } else if self.repeat == RepeatMode::All {
                        if let Some(&last) = self.shuffled_indices.last() {
                            self.play_index(last);
                            return;
                        }
                    } else {
                        self.engine.seek(Duration::ZERO);
                        return;
                    }
                }
            } else if let Some(&last) = self.shuffled_indices.last() {
                self.play_index(last);
                return;
            }
        }

        // Sequential (non-shuffle) playback
        if let Some(curr) = self.current_index {
            if curr > 0 {
                self.play_index(curr - 1);
            } else if self.repeat == RepeatMode::All {
                self.play_index(self.queue.len() - 1);
            } else {
                self.engine.seek(Duration::ZERO);
            }
        } else if !self.queue.is_empty() {
            self.play_index(0);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            self.rebuild_shuffle();
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
            return;
        }
        let mut indices: Vec<usize> = (0..count).collect();
        let mut rng = rand::thread_rng();
        indices.shuffle(&mut rng);

        if let Some(curr) = self.current_index {
            if let Some(pos) = indices.iter().position(|&x| x == curr) {
                indices.remove(pos);
                indices.insert(0, curr);
            }
        }

        self.shuffled_indices = indices;
    }

    pub fn peek_next_index(&self) -> Option<usize> {
        if self.queue.is_empty() {
            return None;
        }

        if self.shuffle {
            if self.shuffled_indices.is_empty() {
                return None;
            }
            if let Some(curr) = self.current_index {
                if let Some(pos) = self.shuffled_indices.iter().position(|&x| x == curr) {
                    if pos + 1 < self.shuffled_indices.len() {
                        return Some(self.shuffled_indices[pos + 1]);
                    } else if self.repeat == RepeatMode::All {
                        return self.shuffled_indices.first().copied();
                    }
                }
            } else {
                return self.shuffled_indices.first().copied();
            }
        } else if let Some(curr) = self.current_index {
            if curr + 1 < self.queue.len() {
                return Some(curr + 1);
            } else if self.repeat == RepeatMode::All {
                return Some(0);
            }
        } else {
            return Some(0);
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

        // Previous from Track 3 goes back to Track 2
        player.current_index = Some(2);
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

        // Next at end wraps to 0
        player.next();
        assert_eq!(player.current_index, Some(0));

        // Prev at 0 wraps to end
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
        assert_eq!(player.peek_next_index(), Some(2));

        player.current_index = Some(2);
        assert_eq!(player.peek_next_index(), None);

        player.repeat = RepeatMode::All;
        assert_eq!(player.peek_next_index(), Some(0));
    }
}
