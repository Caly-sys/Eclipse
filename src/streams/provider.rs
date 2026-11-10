use serde::{Deserialize, Serialize};

/// Represents an active or selected playback audio source.
/// `Local`: The built-in Rodio/Symphonia lossless audio engine playing local files.
/// `Stream(id)`: Remote streaming service (e.g. "spotify", "youtube").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaybackSource {
    #[default]
    Local,
    Stream(String),
}

impl PlaybackSource {
    #[inline]
    pub fn is_local(&self) -> bool {
        matches!(self, PlaybackSource::Local)
    }

    #[inline]
    pub fn is_stream(&self) -> bool {
        matches!(self, PlaybackSource::Stream(_))
    }

    #[inline]
    pub fn provider_id(&self) -> Option<&str> {
        match self {
            PlaybackSource::Local => None,
            PlaybackSource::Stream(id) => Some(id.as_str()),
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            PlaybackSource::Local => "Local Audio",
            PlaybackSource::Stream(p) => match p.to_lowercase().as_str() {
                "spotify" => "Spotify",
                "youtube" | "ytmusic" | "youtube music" => "YouTube Music",
                _ => p.as_str(),
            },
        }
    }
}

/// Unified streaming track metadata representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StreamTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub art_url: Option<String>,
    pub uri: String,
    pub provider: String,
    #[serde(default)]
    pub local_track_id: Option<i64>,
    #[serde(default)]
    pub local_path: Option<String>,
}

impl StreamTrack {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        artist: impl Into<String>,
        album: impl Into<String>,
        duration_secs: f64,
        art_url: Option<String>,
        uri: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            artist: artist.into(),
            album: album.into(),
            duration_secs,
            art_url,
            uri: uri.into(),
            provider: provider.into(),
            local_track_id: None,
            local_path: None,
        }
    }

    #[inline]
    pub fn is_available_locally(&self) -> bool {
        self.local_track_id.is_some() || self.local_path.is_some()
    }

    pub fn set_local_match(&mut self, track_id: i64, path: impl Into<String>) {
        self.local_track_id = Some(track_id);
        self.local_path = Some(path.into());
    }

    pub fn clear_local_match(&mut self) {
        self.local_track_id = None;
        self.local_path = None;
    }

    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            "Unknown Title"
        } else {
            &self.title
        }
    }

    pub fn display_artist(&self) -> &str {
        if self.artist.trim().is_empty() {
            "Unknown Artist"
        } else {
            &self.artist
        }
    }

    pub fn display_album(&self) -> &str {
        if self.album.trim().is_empty() {
            "Unknown Album"
        } else {
            &self.album
        }
    }

    pub fn formatted_duration(&self) -> String {
        let total_secs = self.duration_secs.round() as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}:{:02}", mins, secs)
    }
}

/// Unified streaming playlist representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamPlaylist {
    pub id: String,
    pub name: String,
    pub track_count: usize,
    pub description: Option<String>,
    pub art_url: Option<String>,
}

impl StreamPlaylist {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        track_count: usize,
        description: Option<String>,
        art_url: Option<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            track_count,
            description,
            art_url,
        }
    }
}

/// Unified streaming playback state polled from remote provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StreamPlaybackState {
    pub is_playing: bool,
    pub current_track: Option<StreamTrack>,
    pub position_secs: f64,
    pub duration_secs: f64,
    pub device_name: Option<String>,
    pub is_premium: bool,
    pub volume: f32,
}

impl StreamPlaybackState {
    pub fn progress_pct(&self) -> f32 {
        if self.duration_secs <= 0.0 {
            0.0
        } else {
            (self.position_secs / self.duration_secs).clamp(0.0, 1.0) as f32
        }
    }
}

/// Abstract stream provider contract for third-party streaming services.
pub trait StreamProvider: Send + Sync {
    /// Unique provider identifier (e.g. "spotify", "youtube").
    fn id(&self) -> &'static str;

    /// Human-readable provider name (e.g. "Spotify", "YouTube Music").
    fn name(&self) -> &'static str;

    /// Whether the provider currently has an active, authenticated connection.
    fn is_connected(&self) -> bool;

    /// Establish connection or authenticate with the service.
    fn connect(&mut self) -> Result<(), String>;

    /// Disconnect, sign out, or drop local session tokens.
    fn disconnect(&mut self) -> Result<(), String>;

    /// Search the catalog for tracks matching `query`.
    fn search(&self, query: &str) -> Result<Vec<StreamTrack>, String>;

    /// Fetch the authenticated user's playlists.
    fn get_user_playlists(&self) -> Result<Vec<StreamPlaylist>, String>;

    /// Fetch tracks within a specific playlist.
    fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<StreamTrack>, String>;

    /// Start playback of a specific streaming track.
    fn play(&mut self, track: &StreamTrack) -> Result<(), String>;

    /// Pause active playback.
    fn pause(&mut self) -> Result<(), String>;

    /// Resume paused playback.
    fn resume(&mut self) -> Result<(), String>;

    /// Seek to a specific position in seconds.
    fn seek(&mut self, position_secs: f64) -> Result<(), String>;

    /// Skip to the next track.
    fn next(&mut self) -> Result<(), String>;

    /// Skip to the previous track.
    fn prev(&mut self) -> Result<(), String>;

    /// Set volume (0.0 to 1.0).
    fn set_volume(&mut self, volume: f32) -> Result<(), String>;

    /// Poll current playback state (typically polled at 1Hz or upon event).
    fn get_playback_state(&self) -> Result<Option<StreamPlaybackState>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_source_properties() {
        let local = PlaybackSource::Local;
        assert!(local.is_local());
        assert!(!local.is_stream());
        assert_eq!(local.provider_id(), None);
        assert_eq!(local.display_name(), "Local Audio");

        let spotify = PlaybackSource::Stream("spotify".to_string());
        assert!(!spotify.is_local());
        assert!(spotify.is_stream());
        assert_eq!(spotify.provider_id(), Some("spotify"));
        assert_eq!(spotify.display_name(), "Spotify");

        let yt = PlaybackSource::Stream("youtube".to_string());
        assert_eq!(yt.display_name(), "YouTube Music");

        let custom = PlaybackSource::Stream("custom_service".to_string());
        assert_eq!(custom.display_name(), "custom_service");
    }

    #[test]
    fn test_stream_track_helpers_and_formatting() {
        let track = StreamTrack::new(
            "sp_123",
            "Starboy",
            "The Weeknd",
            "Starboy Album",
            230.4,
            Some("https://example.com/art.jpg".to_string()),
            "spotify:track:sp_123",
            "spotify",
        );

        assert_eq!(track.display_title(), "Starboy");
        assert_eq!(track.display_artist(), "The Weeknd");
        assert_eq!(track.display_album(), "Starboy Album");
        assert_eq!(track.formatted_duration(), "3:50");

        let empty_track = StreamTrack::new(
            "empty",
            "   ",
            "",
            " ",
            0.0,
            None,
            "",
            "spotify",
        );
        assert_eq!(empty_track.display_title(), "Unknown Title");
        assert_eq!(empty_track.display_artist(), "Unknown Artist");
        assert_eq!(empty_track.display_album(), "Unknown Album");
        assert_eq!(empty_track.formatted_duration(), "0:00");
    }

    #[test]
    fn test_stream_playlist_creation() {
        let playlist = StreamPlaylist::new(
            "pl_1",
            "Top Hits",
            50,
            Some("Popular tracks".to_string()),
            Some("https://example.com/pl.jpg".to_string()),
        );

        assert_eq!(playlist.id, "pl_1");
        assert_eq!(playlist.name, "Top Hits");
        assert_eq!(playlist.track_count, 50);
        assert_eq!(playlist.description.as_deref(), Some("Popular tracks"));
    }

    #[test]
    fn test_stream_playback_state_progress() {
        let mut state = StreamPlaybackState::default();
        assert_eq!(state.progress_pct(), 0.0);

        state.duration_secs = 200.0;
        state.position_secs = 50.0;
        assert!((state.progress_pct() - 0.25).abs() < 1e-4);

        state.position_secs = 300.0; // Over duration
        assert_eq!(state.progress_pct(), 1.0);
    }

    #[test]
    fn test_serde_roundtrips() {
        let track = StreamTrack::new(
            "id_1",
            "Title",
            "Artist",
            "Album",
            180.0,
            None,
            "uri",
            "spotify",
        );
        let json = serde_json::to_string(&track).expect("serialize track");
        let deserialized: StreamTrack = serde_json::from_str(&json).expect("deserialize track");
        assert_eq!(track, deserialized);

        let source = PlaybackSource::Stream("spotify".to_string());
        let json_src = serde_json::to_string(&source).expect("serialize source");
        let deser_src: PlaybackSource = serde_json::from_str(&json_src).expect("deserialize source");
        assert_eq!(source, deser_src);
    }

    #[test]
    fn test_stream_track_local_availability() {
        let mut track = StreamTrack::new(
            "id_1",
            "Song",
            "Artist",
            "Album",
            200.0,
            None,
            "spotify:track:123",
            "spotify",
        );
        assert!(!track.is_available_locally());
        assert_eq!(track.local_track_id, None);
        assert_eq!(track.local_path, None);

        track.set_local_match(42, "C:/music/song.flac");
        assert!(track.is_available_locally());
        assert_eq!(track.local_track_id, Some(42));
        assert_eq!(track.local_path.as_deref(), Some("C:/music/song.flac"));

        track.clear_local_match();
        assert!(!track.is_available_locally());
        assert_eq!(track.local_track_id, None);
        assert_eq!(track.local_path, None);
    }
}
