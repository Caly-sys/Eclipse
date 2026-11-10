//! Official YouTube IFrame Player & Lazy YouTube Data API Matcher (ADR-003).
//!
//! Provides:
//! 1. Lazy YouTube resolution with SQLite match cache (`stream_match_cache` table).
//! 2. Background pre-resolution of the next queue track.
//! 3. Alternative match search and selection (`search_youtube_alternatives`).
//! 4. Dedicated small Webview window using `wry` + `tao` for official YouTube embedded playback.
//! 5. Bi-directional IPC (JS -> Rust -> 60 FPS seekbar & track end advancement).
//! 6. ZERO yt-dlp, ZERO ytmusicapi, ZERO scraping, ZERO audio extraction/downloading.

use crate::library::db::Database;
use crate::streams::provider::{StreamPlaybackState, StreamTrack};
use crate::streams::youtube::{
    parse_iso8601_duration, parse_track_title_artist, unescape_html,
    YouTubeSearchListResponse, YouTubeVideoListResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
#[cfg(not(test))]
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";
pub const YOUTUBE_WATCH_BASE: &str = "https://www.youtube.com/watch?v=";

pub const YOUTUBE_QUOTA_ERROR_MESSAGE: &str =
    "YouTube Data API daily quota exhausted (10,000 units/day). Please wait until tomorrow or provide your own API key in Settings > Streaming.";

pub const MISSING_KEY_ERROR_MESSAGE: &str =
    "No YouTube API Key configured. Please enter your YouTube Data API v3 key in Settings > Streaming.";

// ---------------------------------------------------------------------------
// Match Query Generation & Error Detection
// ---------------------------------------------------------------------------

/// Generate a standardized YouTube Data API query from artist and title (`{artist} - {title}`).
pub fn build_match_query(artist: &str, title: &str) -> String {
    let a = artist.trim();
    let t = title.trim();

    if a.is_empty() {
        t.to_string()
    } else if t.is_empty() {
        a.to_string()
    } else {
        format!("{a} - {t}")
    }
}

/// Detect whether an HTTP status code and response body represent a YouTube quota exhaustion error.
pub fn is_quota_exceeded_error(status: u16, body: &str) -> bool {
    let lower = body.to_lowercase();
    status == 403
        && (body.contains("quotaExceeded")
            || body.contains("dailyLimitExceeded")
            || lower.contains("quota"))
}

// ---------------------------------------------------------------------------
// Data Models
// ---------------------------------------------------------------------------

/// An alternative YouTube match candidate shown to the user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YouTubeAlternativeMatch {
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub duration_secs: f64,
    pub thumbnail_url: Option<String>,
}

impl YouTubeAlternativeMatch {
    pub fn new(
        video_id: impl Into<String>,
        title: impl Into<String>,
        channel_title: impl Into<String>,
        duration_secs: f64,
        thumbnail_url: Option<String>,
    ) -> Self {
        Self {
            video_id: video_id.into(),
            title: title.into(),
            channel_title: channel_title.into(),
            duration_secs,
            thumbnail_url,
        }
    }

    /// Convert this match to a standard `StreamTrack`.
    pub fn to_stream_track(&self) -> StreamTrack {
        let (artist, title) = parse_track_title_artist(&self.title, &self.channel_title);
        let uri = format!("{YOUTUBE_WATCH_BASE}{}", self.video_id);
        StreamTrack::new(
            &self.video_id,
            title,
            artist,
            "YouTube Music",
            self.duration_secs,
            self.thumbnail_url.clone(),
            uri,
            "youtube",
        )
    }
}

// ---------------------------------------------------------------------------
// YouTube Data API Search & Match Resolution
// ---------------------------------------------------------------------------

/// Batch-enrich video IDs with accurate durations (ISO 8601) and thumbnails.
pub fn enrich_video_candidates(
    video_ids: &[&str],
    api_key: &str,
) -> HashMap<String, (f64, Option<String>)> {
    let mut map = HashMap::new();
    if video_ids.is_empty() {
        return map;
    }

    for chunk in video_ids.chunks(50) {
        let ids_param = chunk.join(",");
        let url = format!(
            "{YOUTUBE_API_BASE}/videos?part=snippet,contentDetails&id={ids_param}&key={api_key}"
        );

        if let Ok(resp) = ureq::get(&url).timeout(Duration::from_secs(10)).call() {
            if let Ok(video_resp) = resp.into_json::<YouTubeVideoListResponse>() {
                if let Some(items) = video_resp.items {
                    for item in items {
                        let duration = item
                            .content_details
                            .as_ref()
                            .and_then(|cd| cd.duration.as_deref())
                            .map(parse_iso8601_duration)
                            .unwrap_or(0.0);
                        let art_url = item
                            .snippet
                            .as_ref()
                            .and_then(|s| s.thumbnails.as_ref())
                            .and_then(|t| t.best_url());
                        map.insert(item.id, (duration, art_url));
                    }
                }
            }
        }
    }

    map
}

/// Search YouTube Data API v3 for alternative video matches given a search query.
pub fn search_youtube_alternatives(
    api_key: &str,
    query: &str,
    max_results: usize,
) -> Result<Vec<YouTubeAlternativeMatch>, String> {
    let trimmed_key = api_key.trim();
    if trimmed_key.is_empty() {
        return Err(MISSING_KEY_ERROR_MESSAGE.to_string());
    }

    let trimmed_q = query.trim();
    if trimmed_q.is_empty() {
        return Ok(Vec::new());
    }

    let limit = max_results.clamp(1, 25);
    let encoded_query =
        url::form_urlencoded::byte_serialize(trimmed_q.as_bytes()).collect::<String>();
    let url = format!(
        "{YOUTUBE_API_BASE}/search?part=snippet&type=video&videoCategoryId=10&maxResults={limit}&q={encoded_query}&key={trimmed_key}"
    );

    let resp = match ureq::get(&url).timeout(Duration::from_secs(10)).call() {
        Ok(r) => r,
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            if is_quota_exceeded_error(code, &body) {
                return Err(YOUTUBE_QUOTA_ERROR_MESSAGE.to_string());
            }
            return Err(crate::streams::youtube::parse_youtube_error(code, &body));
        }
        Err(ureq::Error::Transport(e)) => {
            return Err(format!("Network error while searching YouTube: {e}"));
        }
    };

    let search_resp = resp
        .into_json::<YouTubeSearchListResponse>()
        .map_err(|e| format!("Failed to parse YouTube search response: {e}"))?;

    let items = search_resp.items.unwrap_or_default();
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let video_ids: Vec<&str> = items
        .iter()
        .filter_map(|it| it.id.video_id.as_deref())
        .collect();

    let details = enrich_video_candidates(&video_ids, trimmed_key);

    let mut results = Vec::with_capacity(items.len());
    for item in items {
        if let Some(vid) = item.id.video_id.as_deref() {
            if let Some(snippet) = item.snippet {
                let raw_title = snippet.title.as_deref().unwrap_or("Unknown Title");
                let title = unescape_html(raw_title);
                let channel_title = unescape_html(
                    snippet
                        .channel_title
                        .as_deref()
                        .unwrap_or("YouTube Music"),
                );
                let (duration, enriched_art) = details.get(vid).cloned().unwrap_or((0.0, None));
                let art = enriched_art.or_else(|| {
                    snippet.thumbnails.as_ref().and_then(|t| t.best_url())
                });

                results.push(YouTubeAlternativeMatch {
                    video_id: vid.to_string(),
                    title,
                    channel_title,
                    duration_secs: duration,
                    thumbnail_url: art,
                });
            }
        }
    }

    Ok(results)
}

/// Resolve a track via YouTube Data API v3 (`search` for `{artist} - {title}`).
pub fn resolve_youtube_match(
    api_key: &str,
    artist: &str,
    title: &str,
) -> Result<YouTubeAlternativeMatch, String> {
    let query = build_match_query(artist, title);
    let alternatives = search_youtube_alternatives(api_key, &query, 5)?;
    alternatives
        .into_iter()
        .next()
        .ok_or_else(|| format!("No YouTube match found for query: '{query}'"))
}

/// Retrieve a cached pairing from SQLite or resolve it on-demand via YouTube Data API.
pub fn resolve_or_cached_match(
    db: &Database,
    api_key: Option<&str>,
    track: &StreamTrack,
) -> Result<YouTubeAlternativeMatch, String> {
    // 1. Check SQLite match cache
    let cache_key = if !track.id.is_empty() {
        &track.id
    } else {
        &track.uri
    };

    if let Ok(Some(cached)) = db.get_stream_match(cache_key) {
        return Ok(YouTubeAlternativeMatch {
            video_id: cached.youtube_video_id,
            title: cached.title,
            channel_title: cached.channel,
            duration_secs: cached.duration_secs,
            thumbnail_url: track.art_url.clone(),
        });
    }

    // 2. Resolve on-demand via YouTube Data API
    let key = api_key
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| MISSING_KEY_ERROR_MESSAGE.to_string())?;

    let matched = resolve_youtube_match(key, &track.artist, &track.title)?;

    // 3. Cache pairing in SQLite for instant future plays
    let _ = db.set_stream_match(
        cache_key,
        &matched.video_id,
        &matched.title,
        &matched.channel_title,
        matched.duration_secs,
    );

    Ok(matched)
}

/// Pre-resolve the NEXT track in queue lazily in the background.
pub fn prefetch_next_track_match(
    db: Arc<Database>,
    api_key: Option<String>,
    next_track: StreamTrack,
) {
    thread::spawn(move || {
        let key = if !next_track.id.is_empty() {
            &next_track.id
        } else {
            &next_track.uri
        };

        // If already in SQLite cache, nothing to do
        if let Ok(Some(_)) = db.get_stream_match(key) {
            return;
        }

        if let Some(ref api_k) = api_key {
            if let Ok(matched) = resolve_youtube_match(api_k, &next_track.artist, &next_track.title) {
                let _ = db.set_stream_match(
                    key,
                    &matched.video_id,
                    &matched.title,
                    &matched.channel_title,
                    matched.duration_secs,
                );
            }
        }
    });
}

/// Select an alternative YouTube match for a given track, updating the SQLite match cache.
pub fn select_alternative_match(
    db: &Database,
    spotify_id: &str,
    alt: &YouTubeAlternativeMatch,
) -> Result<(), String> {
    db.set_stream_match(
        spotify_id,
        &alt.video_id,
        &alt.title,
        &alt.channel_title,
        alt.duration_secs,
    )
    .map_err(|e| format!("Failed to update SQLite match cache: {e}"))
}

// ---------------------------------------------------------------------------
// HTML IFrame Embed Template
// ---------------------------------------------------------------------------

/// Generate the self-contained HTML page embedding the official YouTube IFrame Player API.
pub fn generate_player_html(initial_video_id: &str) -> String {
    // Sanitize video ID to avoid injection
    let clean_id: String = initial_video_id
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Eclipse Player</title>
  <style>
    body, html {{
      margin: 0;
      padding: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
      background-color: #0d0a14;
      display: flex;
      align-items: center;
      justify-content: center;
    }}
    #player {{
      width: 100%;
      height: 100%;
      border: 0;
    }}
  </style>
</head>
<body>
  <div id="player"></div>
  <script>
    var tag = document.createElement('script');
    tag.src = "https://www.youtube.com/iframe_api";
    var firstScriptTag = document.getElementsByTagName('script')[0];
    firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);

    var player = null;
    var timeInterval = null;

    function sendIpc(obj) {{
      try {{
        if (window.ipc && window.ipc.postMessage) {{
          window.ipc.postMessage(JSON.stringify(obj));
        }}
      }} catch (err) {{
        console.error("IPC send error:", err);
      }}
    }}

    function onYouTubeIframeAPIReady() {{
      player = new YT.Player('player', {{
        height: '100%',
        width: '100%',
        videoId: '{clean_id}',
        playerVars: {{
          'playsinline': 1,
          'autoplay': 1,
          'controls': 1,
          'enablejsapi': 1,
          'origin': 'https://www.youtube.com',
          'modestbranding': 1,
          'rel': 0
        }},
        events: {{
          'onReady': onPlayerReady,
          'onStateChange': onPlayerStateChange,
          'onError': onPlayerError
        }}
      }});
    }}

    function onPlayerReady(event) {{
      sendIpc({{ type: 'ready' }});
      if (timeInterval) clearInterval(timeInterval);
      timeInterval = setInterval(function() {{
        if (player && typeof player.getCurrentTime === 'function' && typeof player.getDuration === 'function') {{
          var cur = player.getCurrentTime() || 0;
          var dur = player.getDuration() || 0;
          sendIpc({{ type: 'timeupdate', currentTime: cur, duration: dur }});
        }}
      }}, 250);
    }}

    function onPlayerStateChange(event) {{
      // YT.PlayerState: -1 (unstarted), 0 (ended), 1 (playing), 2 (paused), 3 (buffering), 5 (cued)
      sendIpc({{ type: 'statechange', state: event.data }});
    }}

    function onPlayerError(event) {{
      sendIpc({{ type: 'error', errorCode: event.data }});
    }}
  </script>
</body>
</html>"#
    )
}

// ---------------------------------------------------------------------------
// Webview IPC & Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PlayerWebviewCommand {
    LoadVideo { video_id: String, start_secs: f64 },
    Play,
    Pause,
    Seek(f64),
    SetVolume(f32),
    SetVisible(bool),
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PlayerIpcMessage {
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "timeupdate")]
    TimeUpdate {
        #[serde(rename = "currentTime")]
        current_time: f64,
        duration: f64,
    },
    #[serde(rename = "statechange")]
    StateChange {
        state: i32,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "errorCode")]
        error_code: i32,
    },
}

// ---------------------------------------------------------------------------
// YouTube Webview Player Controller
// ---------------------------------------------------------------------------

/// Official YouTube Webview Player backed by a dedicated window running `wry` + `tao`.
pub struct YouTubeWebviewPlayer {
    cmd_tx: Option<tao::event_loop::EventLoopProxy<PlayerWebviewCommand>>,
    ipc_rx: Receiver<PlayerIpcMessage>,
    thread_handle: Option<JoinHandle<()>>,
    pub playback_state: Arc<RwLock<StreamPlaybackState>>,
    is_ready: Arc<AtomicBool>,
    is_active: bool,
}

impl Default for YouTubeWebviewPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeWebviewPlayer {
    /// Initialize a new YouTube Webview Player.
    pub fn new() -> Self {
        let (_ipc_tx, ipc_rx) = channel::<PlayerIpcMessage>();
        let playback_state = Arc::new(RwLock::new(StreamPlaybackState::default()));
        let is_ready = Arc::new(AtomicBool::new(false));

        #[cfg(test)]
        {
            Self {
                cmd_tx: None,
                ipc_rx,
                thread_handle: None,
                playback_state,
                is_ready,
                is_active: false,
            }
        }

        #[cfg(not(test))]
        {
            let (proxy_tx, proxy_rx) =
                channel::<Option<tao::event_loop::EventLoopProxy<PlayerWebviewCommand>>>();
            let state_clone = Arc::clone(&playback_state);
            let ready_clone = Arc::clone(&is_ready);

            let thread_handle = thread::Builder::new()
                .name("eclipse-youtube-webview".to_string())
                .spawn(move || {
                    run_webview_event_loop(_ipc_tx, proxy_tx, state_clone, ready_clone);
                })
                .ok();

            // Wait briefly for proxy initialization
            let cmd_tx = proxy_rx.recv_timeout(Duration::from_millis(1500)).ok().flatten();

            Self {
                cmd_tx,
                ipc_rx,
                thread_handle,
                playback_state,
                is_ready,
                is_active: false,
            }
        }
    }

    /// Check if the webview window was initialized and has an active command proxy.
    pub fn is_available(&self) -> bool {
        self.cmd_tx.is_some()
    }

    /// Load and play a YouTube video by ID in the official embedded player.
    pub fn play_video(&mut self, video_id: &str, track: &StreamTrack) -> Result<(), String> {
        let mut state = self.playback_state.write().unwrap();
        state.is_playing = true;
        state.current_track = Some(track.clone());
        state.position_secs = 0.0;
        state.duration_secs = track.duration_secs;
        state.device_name = Some("Embedded YouTube Player".to_string());
        drop(state);

        self.is_active = true;

        if let Some(ref proxy) = self.cmd_tx {
            let cmd = PlayerWebviewCommand::LoadVideo {
                video_id: video_id.to_string(),
                start_secs: 0.0,
            };
            let _ = proxy.send_event(cmd);
            let _ = proxy.send_event(PlayerWebviewCommand::Play);
            Ok(())
        } else {
            // Graceful fallback to default browser if webview initialization failed
            #[cfg(not(test))]
            {
                let url = format!("{YOUTUBE_WATCH_BASE}{video_id}");
                let _ = webbrowser::open(&url);
            }
            Ok(())
        }
    }

    pub fn pause(&self) {
        if let Some(ref proxy) = self.cmd_tx {
            let _ = proxy.send_event(PlayerWebviewCommand::Pause);
        }
        let mut state = self.playback_state.write().unwrap();
        state.is_playing = false;
    }

    pub fn resume(&self) {
        if let Some(ref proxy) = self.cmd_tx {
            let _ = proxy.send_event(PlayerWebviewCommand::Play);
        }
        let mut state = self.playback_state.write().unwrap();
        state.is_playing = true;
    }

    pub fn seek(&self, position_secs: f64) {
        if let Some(ref proxy) = self.cmd_tx {
            let _ = proxy.send_event(PlayerWebviewCommand::Seek(position_secs));
        }
        let mut state = self.playback_state.write().unwrap();
        state.position_secs = position_secs;
    }

    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        if let Some(ref proxy) = self.cmd_tx {
            let _ = proxy.send_event(PlayerWebviewCommand::SetVolume(clamped));
        }
        let mut state = self.playback_state.write().unwrap();
        state.volume = clamped;
    }

    pub fn set_visible(&self, visible: bool) {
        if let Some(ref proxy) = self.cmd_tx {
            let _ = proxy.send_event(PlayerWebviewCommand::SetVisible(visible));
        }
    }

    /// Process incoming IPC messages from the webview.
    /// Returns `true` if the track ended (state == 0).
    pub fn update(&mut self) -> bool {
        let mut track_ended = false;

        while let Ok(msg) = self.ipc_rx.try_recv() {
            match msg {
                PlayerIpcMessage::Ready => {
                    self.is_ready.store(true, Ordering::SeqCst);
                }
                PlayerIpcMessage::TimeUpdate {
                    current_time,
                    duration,
                } => {
                    let mut state = self.playback_state.write().unwrap();
                    state.position_secs = current_time;
                    if duration > 0.0 {
                        state.duration_secs = duration;
                    }
                }
                PlayerIpcMessage::StateChange { state: st } => {
                    let mut state = self.playback_state.write().unwrap();
                    match st {
                        0 => {
                            // Ended
                            state.is_playing = false;
                            state.position_secs = state.duration_secs;
                            track_ended = true;
                        }
                        1 => {
                            // Playing
                            state.is_playing = true;
                        }
                        2 => {
                            // Paused
                            state.is_playing = false;
                        }
                        _ => {}
                    }
                }
                PlayerIpcMessage::Error { error_code } => {
                    eprintln!("YouTube Player IFrame error code: {error_code}");
                }
            }
        }

        track_ended
    }

    /// Shutdown the webview player and background thread.
    pub fn shutdown(&mut self) {
        if let Some(proxy) = self.cmd_tx.take() {
            let _ = proxy.send_event(PlayerWebviewCommand::Shutdown);
        }
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for YouTubeWebviewPlayer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ---------------------------------------------------------------------------
// Webview Background Runner
// ---------------------------------------------------------------------------

#[cfg(not(test))]
fn run_webview_event_loop(
    ipc_tx: Sender<PlayerIpcMessage>,
    proxy_tx: Sender<Option<tao::event_loop::EventLoopProxy<PlayerWebviewCommand>>>,
    _playback_state: Arc<RwLock<StreamPlaybackState>>,
    _is_ready: Arc<AtomicBool>,
) {
    let mut ev_builder = tao::event_loop::EventLoopBuilder::<PlayerWebviewCommand>::with_user_event();
    #[cfg(target_os = "windows")]
    {
        use tao::platform::windows::EventLoopBuilderExtWindows;
        ev_builder.with_any_thread(true);
    }

    let event_loop = ev_builder.build();
    let proxy = event_loop.create_proxy();

    let window_res = tao::window::WindowBuilder::new()
        .with_title("Eclipse - Stream Player")
        .with_inner_size(tao::dpi::LogicalSize::new(480.0, 270.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(320.0, 180.0))
        .with_visible(true)
        .build(&event_loop);

    let window = match window_res {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to create YouTube player window: {e}");
            let _ = proxy_tx.send(None);
            return;
        }
    };

    let html = generate_player_html("");
    let ipc_tx_clone = ipc_tx.clone();

    let webview_res = wry::WebViewBuilder::new()
        .with_html(&html)
        .with_ipc_handler(move |req| {
            let body = req.body();
            if let Ok(msg) = serde_json::from_str::<PlayerIpcMessage>(body) {
                let _ = ipc_tx_clone.send(msg);
            }
        })
        .build(&window);

    let webview = match webview_res {
        Ok(wv) => wv,
        Err(e) => {
            eprintln!("Failed to create YouTube webview: {e}");
            let _ = proxy_tx.send(None);
            return;
        }
    };

    // Send proxy back to constructor
    let _ = proxy_tx.send(Some(proxy));

    use tao::event::{Event, WindowEvent};
    use tao::event_loop::ControlFlow;
    use tao::platform::run_return::EventLoopExtRunReturn;

    let mut event_loop = event_loop;
    let _ = event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(cmd) => match cmd {
                PlayerWebviewCommand::LoadVideo {
                    video_id,
                    start_secs,
                } => {
                    let script = format!(
                        "if (player && typeof player.loadVideoById === 'function') {{ player.loadVideoById({{ videoId: '{}', startSeconds: {} }}); }}",
                        video_id, start_secs
                    );
                    let _ = webview.evaluate_script(&script);
                    window.set_visible(true);
                }
                PlayerWebviewCommand::Play => {
                    let _ = webview.evaluate_script(
                        "if (player && typeof player.playVideo === 'function') { player.playVideo(); }",
                    );
                }
                PlayerWebviewCommand::Pause => {
                    let _ = webview.evaluate_script(
                        "if (player && typeof player.pauseVideo === 'function') { player.pauseVideo(); }",
                    );
                }
                PlayerWebviewCommand::Seek(pos) => {
                    let script = format!(
                        "if (player && typeof player.seekTo === 'function') {{ player.seekTo({}, true); }}",
                        pos
                    );
                    let _ = webview.evaluate_script(&script);
                }
                PlayerWebviewCommand::SetVolume(vol) => {
                    let pct = (vol * 100.0).clamp(0.0, 100.0);
                    let script = format!(
                        "if (player && typeof player.setVolume === 'function') {{ player.setVolume({}); }}",
                        pct
                    );
                    let _ = webview.evaluate_script(&script);
                }
                PlayerWebviewCommand::SetVisible(vis) => {
                    window.set_visible(vis);
                }
                PlayerWebviewCommand::Shutdown => {
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                // Hiding instead of closing application
                window.set_visible(false);
            }
            _ => {}
        }
    });
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_query_generation() {
        assert_eq!(
            build_match_query("Daft Punk", "Get Lucky"),
            "Daft Punk - Get Lucky"
        );
        assert_eq!(
            build_match_query("Queen", "Bohemian Rhapsody"),
            "Queen - Bohemian Rhapsody"
        );
        // Artist with whitespace padding
        assert_eq!(
            build_match_query("  The Weeknd  ", "  Blinding Lights  "),
            "The Weeknd - Blinding Lights"
        );
        // Empty artist fallback
        assert_eq!(build_match_query("", "Starboy"), "Starboy");
        // Empty title fallback
        assert_eq!(build_match_query("Hans Zimmer", ""), "Hans Zimmer");
    }

    #[test]
    fn test_quota_error_detection() {
        let quota_body = r#"{
            "error": {
                "code": 403,
                "message": "The request cannot be completed because you have exceeded your quota.",
                "errors": [{ "reason": "quotaExceeded" }]
            }
        }"#;
        assert!(is_quota_exceeded_error(403, quota_body));

        let daily_limit_body = r#"{
            "error": {
                "code": 403,
                "message": "Daily Limit Exceeded",
                "errors": [{ "reason": "dailyLimitExceeded" }]
            }
        }"#;
        assert!(is_quota_exceeded_error(403, daily_limit_body));

        // Non-403 error
        assert!(!is_quota_exceeded_error(400, quota_body));

        // 403 with other error reason (not quota)
        let forbidden_body = r#"{ "error": { "code": 403, "message": "Forbidden" } }"#;
        assert!(!is_quota_exceeded_error(403, forbidden_body));
    }

    #[test]
    fn test_sqlite_match_cache_save_and_retrieval() -> Result<(), Box<dyn std::error::Error>> {
        let db = Database::open_in_memory()?;
        let spotify_id = "spotify_track_abc123";

        // Verify initially empty
        let initial = db.get_stream_match(spotify_id)?;
        assert!(initial.is_none());

        // Cache pairing
        db.set_stream_match(
            spotify_id,
            "dQw4w9WgXcQ",
            "Rick Astley - Never Gonna Give You Up",
            "Rick Astley",
            212.0,
        )?;

        // Retrieve and verify
        let cached = db.get_stream_match(spotify_id)?.expect("cached record");
        assert_eq!(cached.spotify_id, spotify_id);
        assert_eq!(cached.youtube_video_id, "dQw4w9WgXcQ");
        assert_eq!(cached.title, "Rick Astley - Never Gonna Give You Up");
        assert_eq!(cached.channel, "Rick Astley");
        assert_eq!(cached.duration_secs, 212.0);

        // Update with alternative match
        let alt = YouTubeAlternativeMatch::new(
            "alt_video_id_789",
            "Never Gonna Give You Up (Official Remaster)",
            "RickAstleyVEVO",
            214.0,
            Some("https://i.ytimg.com/vi/alt/hqdefault.jpg".to_string()),
        );

        select_alternative_match(&db, spotify_id, &alt).expect("select alternative");

        let updated = db.get_stream_match(spotify_id)?.expect("updated record");
        assert_eq!(updated.youtube_video_id, "alt_video_id_789");
        assert_eq!(updated.title, "Never Gonna Give You Up (Official Remaster)");
        assert_eq!(updated.channel, "RickAstleyVEVO");
        assert_eq!(updated.duration_secs, 214.0);

        // Delete
        db.delete_stream_match(spotify_id)?;
        assert!(db.get_stream_match(spotify_id)?.is_none());

        Ok(())
    }

    #[test]
    fn test_html_embed_template_rendering() {
        let html = generate_player_html("kJQP7kiw5Fk");

        // Verify YouTube iframe API script tag inclusion
        assert!(html.contains("https://www.youtube.com/iframe_api"));
        // Verify target video ID embedded
        assert!(html.contains("kJQP7kiw5Fk"));
        // Verify enablejsapi=1
        assert!(html.contains("'enablejsapi': 1"));
        // Verify origin param
        assert!(html.contains("'origin': 'https://www.youtube.com'"));
        // Verify IPC messaging
        assert!(html.contains("window.ipc.postMessage"));
        // Verify timeupdate interval
        assert!(html.contains("timeupdate"));
        // Verify statechange listener
        assert!(html.contains("onPlayerStateChange"));
        // Verify dark background styling
        assert!(html.contains("background-color: #0d0a14"));
    }

    #[test]
    fn test_ipc_message_deserialization() {
        let ready_json = r#"{"type":"ready"}"#;
        let msg: PlayerIpcMessage = serde_json::from_str(ready_json).expect("valid ready");
        matches!(msg, PlayerIpcMessage::Ready);

        let time_json = r#"{"type":"timeupdate","currentTime":42.5,"duration":210.0}"#;
        let msg: PlayerIpcMessage = serde_json::from_str(time_json).expect("valid timeupdate");
        if let PlayerIpcMessage::TimeUpdate { current_time, duration } = msg {
            assert_eq!(current_time, 42.5);
            assert_eq!(duration, 210.0);
        } else {
            panic!("Expected TimeUpdate");
        }

        let state_json = r#"{"type":"statechange","state":0}"#;
        let msg: PlayerIpcMessage = serde_json::from_str(state_json).expect("valid statechange");
        if let PlayerIpcMessage::StateChange { state } = msg {
            assert_eq!(state, 0); // Ended
        } else {
            panic!("Expected StateChange");
        }
    }

    #[test]
    fn test_alternative_match_model_conversion() {
        let alt = YouTubeAlternativeMatch::new(
            "vid123",
            "M83 - Midnight City",
            "M83 Music",
            243.0,
            Some("https://example.com/art.jpg".to_string()),
        );

        let track = alt.to_stream_track();
        assert_eq!(track.id, "vid123");
        assert_eq!(track.artist, "M83");
        assert_eq!(track.title, "Midnight City");
        assert_eq!(track.duration_secs, 243.0);
        assert_eq!(track.provider, "youtube");
        assert_eq!(track.uri, "https://www.youtube.com/watch?v=vid123");
    }
}
