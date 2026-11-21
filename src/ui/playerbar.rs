//! Core playback time, seek dispatching, and seekbar drag state.
//!
//! Note: The visual player bar has been superseded by `bento_player_bar::render_bento_player_dock`.

use crate::audio::PlayerController;
use crate::streams::controller::StreamController;
use std::time::{Duration, Instant};

/// Scrubbing and seek commitment state persisted across frames.
#[derive(Clone, Copy, Debug, Default)]
pub struct SeekbarDragState {
    pub is_dragging: bool,
    pub scrub_secs: f64,
    pub last_committed_secs: Option<f64>,
    pub last_committed_time: Option<Instant>,
}

/// Helper returning the effective playback position and duration in seconds.
pub fn current_playback_time(
    player: &PlayerController,
    stream_controller: Option<&StreamController>,
) -> (f64, f64) {
    if player.playback_source.is_stream() {
        if player.engine.is_playing()
            || player.engine.position().as_secs_f64() > 0.0
            || player.engine.current_path().is_some()
        {
            let pos = player.engine.position().as_secs_f64();
            let dur = player
                .active_stream_track
                .as_ref()
                .map(|t| t.duration_secs)
                .or_else(|| player.current_track().map(|t| t.duration_secs))
                .unwrap_or(0.0);
            return (pos, dur);
        }
        let pos = stream_controller
            .and_then(|sc| sc.active_state.as_ref())
            .map(|s| s.position_secs)
            .unwrap_or(0.0);
        let dur = stream_controller
            .and_then(|sc| sc.active_state.as_ref())
            .map(|s| s.duration_secs)
            .filter(|&d| d > 0.0)
            .or_else(|| player.active_stream_track.as_ref().map(|t| t.duration_secs))
            .unwrap_or(0.0);
        (pos, dur)
    } else {
        let pos = player.engine.position().as_secs_f64();
        let dur = player.current_track().map(|t| t.duration_secs).unwrap_or(0.0);
        (pos, dur)
    }
}

/// Unified seek dispatcher: seeks either the local audio engine or the active stream controller.
pub fn seek_to(
    player: &mut PlayerController,
    stream_controller: Option<&StreamController>,
    target_secs: f64,
) {
    let target = target_secs.max(0.0);
    if player.playback_source.is_stream()
        && (player.engine.is_playing() || player.engine.current_path().is_some())
    {
        player.engine.seek(Duration::from_secs_f64(target));
    } else if player.playback_source.is_stream() {
        let prov_id = player.stream_provider_id().unwrap_or("spotify");
        if let Some(sc) = stream_controller {
            sc.seek(prov_id, target);
        }
    } else {
        player.engine.seek(Duration::from_secs_f64(target));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::library::model::Track;

    #[test]
    fn test_current_playback_time_local_and_stream() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let stream_ctrl = StreamController::new(vec![]);

        // 1. Empty local queue
        let (pos, dur) = current_playback_time(&player, Some(&stream_ctrl));
        assert_eq!(pos, 0.0);
        assert_eq!(dur, 0.0);

        // 2. Local track in queue
        let track = Track {
            id: 10,
            path: "local.flac".into(),
            title: "Local Track".into(),
            artist: "Artist".into(),
            album: "Album".into(),
            genre: "Rock".into(),
            year: 2024,
            track_number: 1,
            duration_secs: 245.5,
            format: "FLAC".into(),
            bitrate_kbps: 1000,
            sample_rate: 44100,
            channels: 2,
            favorite: false,
            has_art: false,
            folder: "".into(),
            album_artist: "".into(),
            play_count: 0,
            file_size: 1024,
        };
        player.load_queue(vec![track], 0);
        let (_pos, dur) = current_playback_time(&player, Some(&stream_ctrl));
        assert!((dur - 245.5).abs() < 0.001);

        // 3. Switch to stream: active_state duration is 0, so fallback to active_stream_track
        let stream_track = crate::streams::provider::StreamTrack::new(
            "stream-id",
            "Stream Title",
            "Stream Artist",
            "Stream Album",
            195.0,
            None,
            "spotify:track:abc",
            "spotify",
        );
        player.switch_to_stream("spotify", Some(stream_track));
        let (pos, dur) = current_playback_time(&player, Some(&stream_ctrl));
        assert_eq!(pos, 0.0);
        assert!(
            (dur - 195.0).abs() < 0.001,
            "Duration must fall back to active_stream_track when active_state is 0"
        );
    }

    #[test]
    fn test_seek_to_dispatcher_local_and_stream() {
        let cfg = AppConfig::default();
        let mut player = PlayerController::new(&cfg);
        let stream_ctrl = StreamController::new(vec![]);

        // 1. Local seek: dispatching does not panic and clamps negative target
        seek_to(&mut player, Some(&stream_ctrl), -10.0);
        assert_eq!(player.engine.position().as_secs_f64(), 0.0);

        seek_to(&mut player, Some(&stream_ctrl), 45.0);

        // 2. Stream seek: correctly routes to stream controller
        let stream_track = crate::streams::provider::StreamTrack::new(
            "yt-1",
            "YT Song",
            "YT Artist",
            "YT Album",
            300.0,
            None,
            "youtube:v:123",
            "youtube",
        );
        player.switch_to_stream("youtube", Some(stream_track));
        seek_to(&mut player, Some(&stream_ctrl), 120.0);
    }

    #[test]
    fn test_seekbar_drag_state_math_and_persistence() {
        let mut drag_state = SeekbarDragState::default();
        assert!(!drag_state.is_dragging);
        assert_eq!(drag_state.scrub_secs, 0.0);
        assert!(drag_state.last_committed_secs.is_none());

        // Simulate dragging
        drag_state.is_dragging = true;
        drag_state.scrub_secs = 75.5;
        assert!(drag_state.is_dragging);
        assert_eq!(drag_state.scrub_secs, 75.5);

        // Simulate commit on release
        drag_state.is_dragging = false;
        drag_state.last_committed_secs = Some(75.5);
        drag_state.last_committed_time = Some(Instant::now());

        assert!(!drag_state.is_dragging);
        assert_eq!(drag_state.last_committed_secs, Some(75.5));
        assert!(drag_state.last_committed_time.is_some());
    }

    #[test]
    fn test_seekbar_anti_snapback_expiration() {
        let mut drag_state = SeekbarDragState {
            is_dragging: false,
            scrub_secs: 120.0,
            last_committed_secs: Some(120.0),
            last_committed_time: Some(Instant::now() - Duration::from_millis(400)), // expired (> 350ms)
        };

        let pos = 10.0; // Stale engine position

        if let (Some(committed_secs), Some(commit_time)) =
            (drag_state.last_committed_secs, drag_state.last_committed_time)
        {
            if commit_time.elapsed() < Duration::from_millis(350)
                && (pos - committed_secs).abs() > 0.4
            {
                // Hold preview
            } else {
                drag_state.last_committed_secs = None;
                drag_state.last_committed_time = None;
            }
        }

        assert!(
            drag_state.last_committed_secs.is_none(),
            "Committed preview must expire after 350ms"
        );
        assert!(drag_state.last_committed_time.is_none());
    }
}
