//! Minimal Spotify provider stub for Eclipse.
//! All developer app, OAuth PKCE, 127.0.0.1 redirect, and Connect player code have been removed.
//! Zero-login public Spotify link metadata importing is handled in `src/streams/spotify_public.rs` (TASK-R2).

use crate::streams::provider::{
    StreamPlaybackState, StreamPlaylist, StreamProvider, StreamTrack,
};

pub const SPOTIFY_PROVIDER_ID: &str = "spotify";
pub const SPOTIFY_PROVIDER_NAME: &str = "Spotify";

/// Stub stream provider for Spotify.
/// Replaces the legacy developer app / PKCE / Connect implementation.
#[derive(Debug, Default, Clone)]
pub struct SpotifyProvider;

impl SpotifyProvider {
    pub fn new() -> Self {
        Self
    }
}

impl StreamProvider for SpotifyProvider {
    fn id(&self) -> &'static str {
        SPOTIFY_PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        SPOTIFY_PROVIDER_NAME
    }

    fn is_connected(&self) -> bool {
        // Zero-login: always available without authentication
        true
    }

    fn connect(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn search(&self, _query: &str) -> Result<Vec<StreamTrack>, String> {
        Ok(Vec::new())
    }

    fn get_user_playlists(&self) -> Result<Vec<StreamPlaylist>, String> {
        Ok(Vec::new())
    }

    fn get_playlist_tracks(&self, _playlist_id: &str) -> Result<Vec<StreamTrack>, String> {
        Ok(Vec::new())
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
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spotify_provider_stub() {
        let mut provider = SpotifyProvider::new();
        assert_eq!(provider.id(), "spotify");
        assert_eq!(provider.name(), "Spotify");
        assert!(provider.is_connected());
        assert!(provider.connect().is_ok());
        assert!(provider.disconnect().is_ok());
        assert_eq!(provider.search("test").unwrap(), Vec::new());
        assert_eq!(provider.get_user_playlists().unwrap(), Vec::new());
        assert_eq!(provider.get_playlist_tracks("pl_1").unwrap(), Vec::new());
        assert!(provider.pause().is_ok());
        assert!(provider.resume().is_ok());
        assert!(provider.seek(10.0).is_ok());
        assert!(provider.next().is_ok());
        assert!(provider.prev().is_ok());
        assert!(provider.set_volume(0.8).is_ok());
        assert_eq!(provider.get_playback_state().unwrap(), None);
    }
}
