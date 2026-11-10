use crate::streams::provider::{
    StreamPlaybackState, StreamPlaylist, StreamProvider, StreamTrack,
};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
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
}

impl StreamController {
    /// Initialize a new stream controller and spawn its background worker.
    pub fn new(providers: Vec<Box<dyn StreamProvider>>) -> Self {
        let (cmd_tx, cmd_rx) = channel::<StreamCommand>();
        let (event_tx, event_rx) = channel::<StreamEvent>();

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
            connected_providers: HashMap::new(),
            is_busy: false,
            last_error: None,
        }
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

    pub fn next(&self, provider_id: &str) {
        let _ = self.send(StreamCommand::Next {
            provider_id: provider_id.to_string(),
        });
    }

    pub fn prev(&self, provider_id: &str) {
        let _ = self.send(StreamCommand::Prev {
            provider_id: provider_id.to_string(),
        });
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

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            StreamCommand::Shutdown => break,
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
                if let Some(&idx) = provider_map.get(&id_key) {
                    let _ = providers[idx].pause();
                }
            }
            StreamCommand::Resume { provider_id } => {
                let id_key = provider_id.to_lowercase();
                if let Some(&idx) = provider_map.get(&id_key) {
                    let _ = providers[idx].resume();
                }
            }
            StreamCommand::Seek {
                provider_id,
                position_secs,
            } => {
                let id_key = provider_id.to_lowercase();
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
                if let Some(&idx) = provider_map.get(&id_key) {
                    let _ = providers[idx].set_volume(volume);
                }
            }
            StreamCommand::PollState { provider_id } => {
                let id_key = provider_id.to_lowercase();
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
}
