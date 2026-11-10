use crate::streams::credentials::CredentialStore;
use crate::streams::provider::{
    StreamPlaybackState, StreamPlaylist, StreamProvider, StreamTrack,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

pub const YOUTUBE_PROVIDER_ID: &str = "youtube";
pub const YOUTUBE_PROVIDER_NAME: &str = "YouTube Music";
pub const YOUTUBE_API_BASE: &str = "https://www.googleapis.com/youtube/v3";
pub const YOUTUBE_WATCH_BASE: &str = "https://www.youtube.com/watch?v=";
pub const YOUTUBE_EMBED_BASE: &str = "https://www.youtube.com/embed/";

pub const MISSING_KEY_ERROR: &str =
    "No YouTube API Key configured. Please enter your YouTube Data API v3 key in Settings.";
pub const QUOTA_EXCEEDED_ERROR: &str =
    "YouTube Data API daily quota exceeded (10,000 units/day). Please wait until tomorrow or provide your own Google Cloud API Key in Settings.";
pub const KEY_INVALID_ERROR: &str =
    "Invalid YouTube API Key. Please verify your Google Cloud API key in Eclipse Settings.";
pub const ACCESS_NOT_CONFIGURED_ERROR: &str =
    "YouTube Data API v3 is not enabled in your Google Cloud Project. Please enable it in the Google Cloud Console.";

/// Official YouTube Data API v3 integration supporting search, playlist browsing,
/// ISO 8601 duration enrichment, and official embedded browser playback (ADR-003).
pub struct YouTubeProvider {
    api_key: Arc<RwLock<Option<String>>>,
    connected: Arc<RwLock<bool>>,
    playback_state: Arc<RwLock<StreamPlaybackState>>,
    base_timestamp: Arc<RwLock<Instant>>,
    base_position_secs: Arc<RwLock<f64>>,
    volume: Arc<RwLock<f32>>,
}

impl Default for YouTubeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl YouTubeProvider {
    /// Initialize a new YouTubeProvider, loading any stored API key from the OS CredentialStore.
    pub fn new() -> Self {
        let stored_key = CredentialStore::get_youtube_api_key().ok().flatten();
        let has_key = stored_key.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false);

        Self {
            api_key: Arc::new(RwLock::new(stored_key)),
            connected: Arc::new(RwLock::new(has_key)),
            playback_state: Arc::new(RwLock::new(StreamPlaybackState::default())),
            base_timestamp: Arc::new(RwLock::new(Instant::now())),
            base_position_secs: Arc::new(RwLock::new(0.0)),
            volume: Arc::new(RwLock::new(1.0)),
        }
    }

    /// Builder helper to set an explicit API key (useful for tests and dynamic configuration).
    pub fn with_api_key(self, api_key: impl Into<String>) -> Self {
        let key = api_key.into();
        let has_key = !key.trim().is_empty();
        *self.api_key.write().unwrap() = Some(key);
        *self.connected.write().unwrap() = has_key;
        self
    }

    /// Set and securely persist the YouTube API Key in the Windows Credential Manager.
    pub fn set_api_key(&self, key: &str) -> Result<(), String> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            self.clear_api_key()
        } else {
            CredentialStore::save_youtube_api_key(trimmed)?;
            *self.api_key.write().unwrap() = Some(trimmed.to_string());
            *self.connected.write().unwrap() = true;
            Ok(())
        }
    }

    /// Retrieve active YouTube API key from memory or OS CredentialStore.
    pub fn get_api_key(&self) -> Option<String> {
        if let Some(key) = self.api_key.read().unwrap().as_ref() {
            if !key.trim().is_empty() {
                return Some(key.clone());
            }
        }
        if let Ok(Some(stored)) = CredentialStore::get_youtube_api_key() {
            if !stored.trim().is_empty() {
                *self.api_key.write().unwrap() = Some(stored.clone());
                *self.connected.write().unwrap() = true;
                return Some(stored);
            }
        }
        None
    }

    /// Clear the stored YouTube API Key from memory and OS CredentialStore.
    pub fn clear_api_key(&self) -> Result<(), String> {
        CredentialStore::clear_youtube_credentials()?;
        *self.api_key.write().unwrap() = None;
        *self.connected.write().unwrap() = false;
        Ok(())
    }

    /// Validate an API key against the YouTube Data API v3 categories endpoint.
    pub fn validate_api_key(&self, key: &str) -> Result<(), String> {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            return Err(MISSING_KEY_ERROR.to_string());
        }

        let url = format!(
            "{YOUTUBE_API_BASE}/videoCategories?part=snippet&regionCode=US&key={trimmed}"
        );

        match ureq::get(&url)
            .timeout(Duration::from_secs(10))
            .call()
        {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                Err(parse_youtube_error(code, &body))
            }
            Err(ureq::Error::Transport(e)) => {
                Err(format!("Network error while validating YouTube API key: {e}"))
            }
        }
    }

    /// Batch enrich video IDs with accurate durations (ISO 8601) and high-res thumbnails.
    pub fn enrich_videos(
        &self,
        video_ids: &[&str],
        api_key: &str,
    ) -> HashMap<String, (f64, Option<String>)> {
        let mut map = HashMap::new();
        if video_ids.is_empty() {
            return map;
        }

        // YouTube videos endpoint supports up to 50 comma-separated IDs per request
        for chunk in video_ids.chunks(50) {
            let ids_param = chunk.join(",");
            let url = format!(
                "{YOUTUBE_API_BASE}/videos?part=snippet,contentDetails&id={ids_param}&key={api_key}"
            );

            let res = ureq::get(&url)
                .timeout(Duration::from_secs(10))
                .call();

            if let Ok(resp) = res {
                if let Ok(video_resp) = resp.into_json::<YouTubeVideoListResponse>() {
                    if let Some(items) = video_resp.items {
                        for item in items {
                            let duration = item
                                .content_details
                                .as_ref()
                                .and_then(|cd| cd.duration.as_deref())
                                .map(parse_iso8601_duration)
                                .unwrap_or(0.0);
                            let art_url = item.snippet.as_ref().and_then(|s| s.thumbnails.as_ref()).and_then(|t| t.best_url());
                            map.insert(item.id, (duration, art_url));
                        }
                    }
                }
            }
        }

        map
    }

    /// Search playlists matching `query`.
    pub fn search_playlists(&self, query: &str) -> Result<Vec<StreamPlaylist>, String> {
        let api_key = match self.get_api_key() {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Err(MISSING_KEY_ERROR.to_string()),
        };

        let encoded_query = url::form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
        let url = format!(
            "{YOUTUBE_API_BASE}/search?part=snippet&type=playlist&maxResults=25&q={encoded_query}&key={api_key}"
        );

        let resp = match ureq::get(&url)
            .timeout(Duration::from_secs(10))
            .call()
        {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                return Err(parse_youtube_error(code, &body));
            }
            Err(ureq::Error::Transport(e)) => {
                return Err(format!("Network error while searching YouTube playlists: {e}"));
            }
        };

        let search_resp = resp
            .into_json::<YouTubeSearchListResponse>()
            .map_err(|e| format!("Failed to parse YouTube playlist search response: {e}"))?;

        let mut playlists = Vec::new();
        if let Some(items) = search_resp.items {
            for item in items {
                if let Some(playlist) = parse_playlist_item(&item) {
                    playlists.push(playlist);
                }
            }
        }

        Ok(playlists)
    }
}

impl StreamProvider for YouTubeProvider {
    fn id(&self) -> &'static str {
        YOUTUBE_PROVIDER_ID
    }

    fn name(&self) -> &'static str {
        YOUTUBE_PROVIDER_NAME
    }

    fn is_connected(&self) -> bool {
        *self.connected.read().unwrap() && self.get_api_key().is_some()
    }

    fn connect(&mut self) -> Result<(), String> {
        match self.get_api_key() {
            Some(k) if !k.trim().is_empty() => {
                *self.connected.write().unwrap() = true;
                Ok(())
            }
            _ => {
                *self.connected.write().unwrap() = false;
                Err(MISSING_KEY_ERROR.to_string())
            }
        }
    }

    fn disconnect(&mut self) -> Result<(), String> {
        *self.connected.write().unwrap() = false;
        let _ = self.clear_api_key();
        let mut state = self.playback_state.write().unwrap();
        *state = StreamPlaybackState::default();
        Ok(())
    }

    fn search(&self, query: &str) -> Result<Vec<StreamTrack>, String> {
        let api_key = match self.get_api_key() {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Err(MISSING_KEY_ERROR.to_string()),
        };

        let trimmed_query = query.trim();
        if trimmed_query.is_empty() {
            return Ok(Vec::new());
        }

        let encoded_query = url::form_urlencoded::byte_serialize(trimmed_query.as_bytes()).collect::<String>();
        let url = format!(
            "{YOUTUBE_API_BASE}/search?part=snippet&type=video&videoCategoryId=10&maxResults=25&q={encoded_query}&key={api_key}"
        );

        let resp = match ureq::get(&url)
            .timeout(Duration::from_secs(10))
            .call()
        {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                return Err(parse_youtube_error(code, &body));
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

        // Collect video IDs for duration enrichment
        let video_ids: Vec<&str> = items
            .iter()
            .filter_map(|item| item.id.video_id.as_deref())
            .collect();

        let details = self.enrich_videos(&video_ids, &api_key);

        let mut tracks = Vec::with_capacity(items.len());
        for item in items {
            if let Some(vid) = item.id.video_id.as_deref() {
                let (duration, enriched_art) = details.get(vid).cloned().unwrap_or((0.0, None));
                if let Some(mut track) = parse_search_track_item(&item, duration) {
                    if let Some(art) = enriched_art {
                        track.art_url = Some(art);
                    }
                    tracks.push(track);
                }
            }
        }

        Ok(tracks)
    }

    fn get_user_playlists(&self) -> Result<Vec<StreamPlaylist>, String> {
        // Without user OAuth, we provide curated music playlists based on popular music charts
        self.search_playlists("Top Music Hits")
    }

    fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<StreamTrack>, String> {
        let api_key = match self.get_api_key() {
            Some(k) if !k.trim().is_empty() => k,
            _ => return Err(MISSING_KEY_ERROR.to_string()),
        };

        let url = format!(
            "{YOUTUBE_API_BASE}/playlistItems?part=snippet,contentDetails&playlistId={playlist_id}&maxResults=50&key={api_key}"
        );

        let resp = match ureq::get(&url)
            .timeout(Duration::from_secs(10))
            .call()
        {
            Ok(r) => r,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                return Err(parse_youtube_error(code, &body));
            }
            Err(ureq::Error::Transport(e)) => {
                return Err(format!("Network error while fetching YouTube playlist tracks: {e}"));
            }
        };

        let playlist_resp = resp
            .into_json::<YouTubePlaylistItemListResponse>()
            .map_err(|e| format!("Failed to parse YouTube playlist items response: {e}"))?;

        let items = playlist_resp.items.unwrap_or_default();
        if items.is_empty() {
            return Ok(Vec::new());
        }

        // Collect video IDs
        let video_ids: Vec<&str> = items
            .iter()
            .filter_map(|item| {
                item.content_details
                    .as_ref()
                    .and_then(|cd| cd.video_id.as_deref())
                    .or_else(|| {
                        item.snippet
                            .as_ref()
                            .and_then(|s| s.resource_id.as_ref())
                            .and_then(|r| r.video_id.as_deref())
                    })
            })
            .collect();

        let details = self.enrich_videos(&video_ids, &api_key);

        let mut tracks = Vec::with_capacity(items.len());
        for item in items {
            let vid_opt = item
                .content_details
                .as_ref()
                .and_then(|cd| cd.video_id.as_deref())
                .or_else(|| {
                    item.snippet
                        .as_ref()
                        .and_then(|s| s.resource_id.as_ref())
                        .and_then(|r| r.video_id.as_deref())
                });

            if let Some(vid) = vid_opt {
                let (duration, enriched_art) = details.get(vid).cloned().unwrap_or((0.0, None));
                if let Some(mut track) = parse_playlist_track_entry(&item, duration) {
                    if let Some(art) = enriched_art {
                        track.art_url = Some(art);
                    }
                    tracks.push(track);
                }
            }
        }

        Ok(tracks)
    }

    fn play(&mut self, track: &StreamTrack) -> Result<(), String> {
        let watch_url = if track.uri.starts_with("http://") || track.uri.starts_with("https://") {
            track.uri.clone()
        } else {
            format!("{YOUTUBE_WATCH_BASE}{}", track.id)
        };

        // Open official YouTube player in default browser off-thread (ADR-003)
        let url_to_open = watch_url;
        thread::spawn(move || {
            let _ = webbrowser::open(&url_to_open);
        });

        // Update local playback state
        let now = Instant::now();
        let volume = *self.volume.read().unwrap();
        let mut state = self.playback_state.write().unwrap();
        *state = StreamPlaybackState {
            is_playing: true,
            current_track: Some(track.clone()),
            position_secs: 0.0,
            duration_secs: track.duration_secs,
            device_name: Some("Web Browser (YouTube Player)".to_string()),
            is_premium: false,
            volume,
        };

        *self.base_timestamp.write().unwrap() = now;
        *self.base_position_secs.write().unwrap() = 0.0;

        Ok(())
    }

    fn pause(&mut self) -> Result<(), String> {
        let mut state = self.playback_state.write().unwrap();
        if state.is_playing {
            let elapsed = self.base_timestamp.read().unwrap().elapsed().as_secs_f64();
            let new_pos = (*self.base_position_secs.read().unwrap() + elapsed)
                .min(state.duration_secs.max(0.0));
            *self.base_position_secs.write().unwrap() = new_pos;
            state.position_secs = new_pos;
            state.is_playing = false;
        }
        Ok(())
    }

    fn resume(&mut self) -> Result<(), String> {
        let mut state = self.playback_state.write().unwrap();
        if !state.is_playing && state.current_track.is_some() {
            state.is_playing = true;
            *self.base_timestamp.write().unwrap() = Instant::now();
        }
        Ok(())
    }

    fn seek(&mut self, position_secs: f64) -> Result<(), String> {
        let mut state = self.playback_state.write().unwrap();
        let clamped = position_secs.clamp(0.0, state.duration_secs.max(0.0));
        *self.base_position_secs.write().unwrap() = clamped;
        *self.base_timestamp.write().unwrap() = Instant::now();
        state.position_secs = clamped;
        Ok(())
    }

    fn next(&mut self) -> Result<(), String> {
        // No remote playlist queue control for browser-hosted YouTube playback
        Ok(())
    }

    fn prev(&mut self) -> Result<(), String> {
        // No remote playlist queue control for browser-hosted YouTube playback
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        let clamped = volume.clamp(0.0, 1.0);
        *self.volume.write().unwrap() = clamped;
        let mut state = self.playback_state.write().unwrap();
        state.volume = clamped;
        Ok(())
    }

    fn get_playback_state(&self) -> Result<Option<StreamPlaybackState>, String> {
        let state_guard = self.playback_state.read().unwrap();
        if state_guard.current_track.is_none() {
            return Ok(None);
        }

        let mut state = state_guard.clone();
        if state.is_playing {
            let elapsed = self.base_timestamp.read().unwrap().elapsed().as_secs_f64();
            let current_pos = *self.base_position_secs.read().unwrap() + elapsed;
            if state.duration_secs > 0.0 && current_pos >= state.duration_secs {
                state.position_secs = state.duration_secs;
                state.is_playing = false;
            } else {
                state.position_secs = current_pos;
            }
        }

        Ok(Some(state))
    }
}

// ---------------------------------------------------------------------------
// ISO 8601 Duration Parser
// ---------------------------------------------------------------------------

/// Parse ISO 8601 duration strings (e.g. `PT3M45S`, `PT1H2M10S`, `PT42S`, `P1DT2H3M4S`) into seconds (`f64`).
pub fn parse_iso8601_duration(input: &str) -> f64 {
    let trimmed = input.trim();
    if !trimmed.starts_with('P') && !trimmed.starts_with('p') {
        return 0.0;
    }

    let rest = &trimmed[1..];
    let (date_part, time_part) = match rest.find(['T', 't']) {
        Some(idx) => (&rest[..idx], Some(&rest[idx + 1..])),
        None => (rest, None),
    };

    let mut total_secs = 0.0;

    // Parse date components: Y, M, W, D
    if !date_part.is_empty() {
        let mut num_buf = String::new();
        for ch in date_part.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_buf.push(ch);
            } else {
                if let Ok(val) = num_buf.parse::<f64>() {
                    match ch {
                        'Y' | 'y' => total_secs += val * 31_536_000.0,
                        'M' | 'm' => total_secs += val * 2_592_000.0,
                        'W' | 'w' => total_secs += val * 604_800.0,
                        'D' | 'd' => total_secs += val * 86_400.0,
                        _ => {}
                    }
                }
                num_buf.clear();
            }
        }
    }

    // Parse time components: H, M, S
    if let Some(time_str) = time_part {
        let mut num_buf = String::new();
        for ch in time_str.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_buf.push(ch);
            } else {
                if let Ok(val) = num_buf.parse::<f64>() {
                    match ch {
                        'H' | 'h' => total_secs += val * 3600.0,
                        'M' | 'm' => total_secs += val * 60.0,
                        'S' | 's' => total_secs += val,
                        _ => {}
                    }
                }
                num_buf.clear();
            }
        }
    }

    total_secs
}

// ---------------------------------------------------------------------------
// HTML Entity Unescaping
// ---------------------------------------------------------------------------

/// Decode HTML entities (e.g. `&amp;` -> `&`, `&#39;` -> `'`, `&quot;` -> `"`, `&#x27;` -> `'`)
/// returned by YouTube Data API v3 titles and descriptions.
pub fn unescape_html(input: &str) -> String {
    if !input.contains('&') {
        return input.to_string();
    }

    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            let mut entity = String::new();
            let mut is_entity = false;

            while let Some(&next_ch) = chars.peek() {
                if next_ch == ';' {
                    chars.next();
                    is_entity = true;
                    break;
                } else if next_ch.is_alphanumeric() || next_ch == '#' {
                    entity.push(chars.next().unwrap());
                    if entity.len() > 10 {
                        break;
                    }
                } else {
                    break;
                }
            }

            if is_entity {
                match entity.as_str() {
                    "amp" => output.push('&'),
                    "quot" => output.push('"'),
                    "apos" => output.push('\''),
                    "lt" => output.push('<'),
                    "gt" => output.push('>'),
                    "nbsp" => output.push(' '),
                    ent if ent.starts_with("#x") || ent.starts_with("#X") => {
                        if let Ok(code) = u32::from_str_radix(&ent[2..], 16) {
                            if let Some(c) = char::from_u32(code) {
                                output.push(c);
                            } else {
                                output.push('&');
                                output.push_str(&entity);
                                output.push(';');
                            }
                        } else {
                            output.push('&');
                            output.push_str(&entity);
                            output.push(';');
                        }
                    }
                    ent if ent.starts_with('#') => {
                        if let Ok(code) = ent[1..].parse::<u32>() {
                            if let Some(c) = char::from_u32(code) {
                                output.push(c);
                            } else {
                                output.push('&');
                                output.push_str(&entity);
                                output.push(';');
                            }
                        } else {
                            output.push('&');
                            output.push_str(&entity);
                            output.push(';');
                        }
                    }
                    _ => {
                        output.push('&');
                        output.push_str(&entity);
                        output.push(';');
                    }
                }
            } else {
                output.push('&');
                output.push_str(&entity);
            }
        } else {
            output.push(ch);
        }
    }

    output
}

// ---------------------------------------------------------------------------
// Title and Artist Extraction
// ---------------------------------------------------------------------------

/// Intelligently extract (artist, title) from video snippet title and channel title.
pub fn parse_track_title_artist(raw_title: &str, channel_title: &str) -> (String, String) {
    let unescaped_raw = unescape_html(raw_title.trim());
    let clean_channel = channel_title
        .trim()
        .strip_suffix(" - Topic")
        .unwrap_or(channel_title.trim());
    let unescaped_channel = unescape_html(clean_channel);

    // Look for standard "Artist - Title" separator (hyphen, en-dash, or em-dash)
    let sep_opt = if unescaped_raw.contains(" - ") {
        Some(" - ")
    } else if unescaped_raw.contains(" – ") {
        Some(" – ")
    } else if unescaped_raw.contains(" — ") {
        Some(" — ")
    } else {
        None
    };

    if let Some(sep) = sep_opt {
        let parts: Vec<&str> = unescaped_raw.splitn(2, sep).collect();
        if parts.len() == 2 {
            let part0 = parts[0].trim();
            let part1 = parts[1].trim();

            if !part0.is_empty() && !part1.is_empty() {
                // If channel matches part1 (e.g. Title - Artist format)
                if unescaped_channel.eq_ignore_ascii_case(part1) {
                    return (part1.to_string(), part0.to_string());
                }
                return (part0.to_string(), part1.to_string());
            }
        }
    }

    let artist = if unescaped_channel.is_empty() {
        "YouTube Music".to_string()
    } else {
        unescaped_channel
    };

    (artist, unescaped_raw)
}

// ---------------------------------------------------------------------------
// Error Categorization
// ---------------------------------------------------------------------------

/// Parse YouTube Data API v3 error response and map into clean, user-friendly messages.
pub fn parse_youtube_error(status: u16, body: &str) -> String {
    let lower_body = body.to_lowercase();

    // Check for Quota Exhaustion
    if status == 403
        && (body.contains("quotaExceeded")
            || body.contains("dailyLimitExceeded")
            || lower_body.contains("quota"))
    {
        return QUOTA_EXCEEDED_ERROR.to_string();
    }

    // Check for Invalid API Key
    if status == 400
        && (body.contains("keyInvalid")
            || lower_body.contains("api key not valid")
            || lower_body.contains("key is invalid"))
    {
        return KEY_INVALID_ERROR.to_string();
    }

    // Check for Service Disabled in Project
    if status == 403 && (body.contains("accessNotConfigured") || lower_body.contains("not enabled"))
    {
        return ACCESS_NOT_CONFIGURED_ERROR.to_string();
    }

    // Attempt to extract error message from standard YouTube JSON error body
    if let Ok(parsed) = serde_json::from_str::<YouTubeErrorResponse>(body) {
        if let Some(details) = parsed.error {
            if let Some(msg) = details.message {
                if !msg.trim().is_empty() {
                    return format!("YouTube API Error ({status}): {msg}");
                }
            }
        }
    }

    // Fallbacks
    match status {
        400 => "YouTube API bad request. Please check your query or settings.".to_string(),
        403 => "YouTube access forbidden. Please verify your API key permissions.".to_string(),
        404 => "YouTube resource not found.".to_string(),
        429 => "YouTube rate limit exceeded. Please wait a moment and try again.".to_string(),
        500..=599 => format!("YouTube service temporarily unavailable ({status}). Please try again later."),
        _ => format!("YouTube API request failed ({status})."),
    }
}

// ---------------------------------------------------------------------------
// YouTube API JSON Data Structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeSearchListResponse {
    pub items: Option<Vec<YouTubeSearchResult>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeSearchResult {
    pub id: YouTubeResourceId,
    pub snippet: Option<YouTubeSnippet>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeResourceId {
    pub kind: Option<String>,
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
    #[serde(rename = "playlistId")]
    pub playlist_id: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeSnippet {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "channelTitle")]
    pub channel_title: Option<String>,
    pub thumbnails: Option<YouTubeThumbnails>,
    #[serde(rename = "resourceId")]
    pub resource_id: Option<YouTubeResourceId>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeThumbnails {
    pub default: Option<YouTubeThumbnailItem>,
    pub medium: Option<YouTubeThumbnailItem>,
    pub high: Option<YouTubeThumbnailItem>,
    pub standard: Option<YouTubeThumbnailItem>,
    pub maxres: Option<YouTubeThumbnailItem>,
}

impl YouTubeThumbnails {
    pub fn best_url(&self) -> Option<String> {
        self.maxres
            .as_ref()
            .or(self.high.as_ref())
            .or(self.medium.as_ref())
            .or(self.standard.as_ref())
            .or(self.default.as_ref())
            .map(|t| t.url.clone())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeThumbnailItem {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeVideoListResponse {
    pub items: Option<Vec<YouTubeVideoItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeVideoItem {
    pub id: String,
    pub snippet: Option<YouTubeSnippet>,
    #[serde(rename = "contentDetails")]
    pub content_details: Option<YouTubeContentDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeContentDetails {
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubePlaylistItemListResponse {
    pub items: Option<Vec<YouTubePlaylistItemEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubePlaylistItemEntry {
    pub id: String,
    pub snippet: Option<YouTubeSnippet>,
    #[serde(rename = "contentDetails")]
    pub content_details: Option<YouTubePlaylistItemContentDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubePlaylistItemContentDetails {
    #[serde(rename = "videoId")]
    pub video_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeErrorResponse {
    pub error: Option<YouTubeErrorDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeErrorDetails {
    pub code: Option<u16>,
    pub message: Option<String>,
    pub errors: Option<Vec<YouTubeErrorItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YouTubeErrorItem {
    pub message: Option<String>,
    pub domain: Option<String>,
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Item Mapping Helpers
// ---------------------------------------------------------------------------

pub fn parse_search_track_item(item: &YouTubeSearchResult, duration_secs: f64) -> Option<StreamTrack> {
    let video_id = item.id.video_id.as_deref()?;
    let snippet = item.snippet.as_ref()?;

    let raw_title = snippet.title.as_deref().unwrap_or("Unknown Title");
    let channel_title = snippet.channel_title.as_deref().unwrap_or("YouTube Music");

    let (artist, title) = parse_track_title_artist(raw_title, channel_title);
    let art_url = snippet.thumbnails.as_ref().and_then(|t| t.best_url());
    let uri = format!("{YOUTUBE_WATCH_BASE}{video_id}");

    Some(StreamTrack::new(
        video_id,
        title,
        artist,
        "YouTube Music",
        duration_secs,
        art_url,
        uri,
        YOUTUBE_PROVIDER_ID,
    ))
}

pub fn parse_playlist_item(item: &YouTubeSearchResult) -> Option<StreamPlaylist> {
    let playlist_id = item.id.playlist_id.as_deref()?;
    let snippet = item.snippet.as_ref()?;

    let name = unescape_html(snippet.title.as_deref().unwrap_or("YouTube Playlist"));
    let description = snippet.description.as_deref().map(unescape_html);
    let art_url = snippet.thumbnails.as_ref().and_then(|t| t.best_url());

    Some(StreamPlaylist::new(
        playlist_id,
        name,
        0,
        description,
        art_url,
    ))
}

pub fn parse_playlist_track_entry(
    entry: &YouTubePlaylistItemEntry,
    duration_secs: f64,
) -> Option<StreamTrack> {
    let video_id = entry
        .content_details
        .as_ref()
        .and_then(|cd| cd.video_id.as_deref())
        .or_else(|| {
            entry
                .snippet
                .as_ref()
                .and_then(|s| s.resource_id.as_ref())
                .and_then(|r| r.video_id.as_deref())
        })?;

    let snippet = entry.snippet.as_ref()?;
    let raw_title = snippet.title.as_deref().unwrap_or("Unknown Title");
    let channel_title = snippet.channel_title.as_deref().unwrap_or("YouTube Music");

    let (artist, title) = parse_track_title_artist(raw_title, channel_title);
    let art_url = snippet.thumbnails.as_ref().and_then(|t| t.best_url());
    let uri = format!("{YOUTUBE_WATCH_BASE}{video_id}");

    Some(StreamTrack::new(
        video_id,
        title,
        artist,
        "YouTube Music",
        duration_secs,
        art_url,
        uri,
        YOUTUBE_PROVIDER_ID,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso8601_duration_various_formats() {
        // Standard PT#M#S
        assert_eq!(parse_iso8601_duration("PT3M45S"), 225.0);
        assert_eq!(parse_iso8601_duration("PT0M30S"), 30.0);

        // Hours + Minutes + Seconds
        assert_eq!(parse_iso8601_duration("PT1H2M10S"), 3730.0);
        assert_eq!(parse_iso8601_duration("PT2H0M0S"), 7200.0);

        // Seconds only
        assert_eq!(parse_iso8601_duration("PT42S"), 42.0);

        // Hours only
        assert_eq!(parse_iso8601_duration("PT1H"), 3600.0);

        // Minutes only
        assert_eq!(parse_iso8601_duration("PT5M"), 300.0);

        // Day + Hour + Minute + Second
        assert_eq!(parse_iso8601_duration("P1DT2H3M4S"), 86400.0 + 7200.0 + 180.0 + 4.0);

        // Fractional seconds
        assert_eq!(parse_iso8601_duration("PT1M20.5S"), 80.5);

        // Zero durations
        assert_eq!(parse_iso8601_duration("PT0S"), 0.0);
        assert_eq!(parse_iso8601_duration("P0D"), 0.0);

        // Lowercase tolerance
        assert_eq!(parse_iso8601_duration("pt3m45s"), 225.0);

        // Invalid or empty inputs safe fallback to 0.0
        assert_eq!(parse_iso8601_duration(""), 0.0);
        assert_eq!(parse_iso8601_duration("invalid"), 0.0);
        assert_eq!(parse_iso8601_duration("P"), 0.0);
    }

    #[test]
    fn test_unescape_html_standard_and_numeric() {
        assert_eq!(
            unescape_html("&quot;Bohemian Rhapsody&quot; - Queen"),
            "\"Bohemian Rhapsody\" - Queen"
        );
        assert_eq!(
            unescape_html("Simon &amp; Garfunkel"),
            "Simon & Garfunkel"
        );
        assert_eq!(
            unescape_html("Don&#39;t Stop Me Now"),
            "Don't Stop Me Now"
        );
        assert_eq!(
            unescape_html("Rock &#x26; Roll"),
            "Rock & Roll"
        );
        assert_eq!(
            unescape_html("Track &lt;Live&gt; &amp; Remastered"),
            "Track <Live> & Remastered"
        );
        assert_eq!(
            unescape_html("Hello&nbsp;World"),
            "Hello World"
        );
        assert_eq!(
            unescape_html("Copyright &#169; 2024"),
            "Copyright © 2024"
        );

        // Plain string without entities returns untouched
        assert_eq!(unescape_html("Midnight City"), "Midnight City");

        // Lone ampersand without entity syntax
        assert_eq!(unescape_html("Tom & Jerry"), "Tom & Jerry");
    }

    #[test]
    fn test_parse_track_title_artist_patterns() {
        // Hyphen separator
        let (artist, title) = parse_track_title_artist("Daft Punk - Get Lucky", "Daft Punk");
        assert_eq!(artist, "Daft Punk");
        assert_eq!(title, "Get Lucky");

        // En-dash separator
        let (artist, title) = parse_track_title_artist("Queen – Bohemian Rhapsody", "Queen Official");
        assert_eq!(artist, "Queen");
        assert_eq!(title, "Bohemian Rhapsody");

        // HTML entities in title
        let (artist, title) = parse_track_title_artist("Guns N&#39; Roses - Sweet Child O&#39; Mine", "Guns N' Roses");
        assert_eq!(artist, "Guns N' Roses");
        assert_eq!(title, "Sweet Child O' Mine");

        // Channel title with "- Topic" stripped
        let (artist, title) = parse_track_title_artist("Starboy", "The Weeknd - Topic");
        assert_eq!(artist, "The Weeknd");
        assert_eq!(title, "Starboy");

        // Inverted "Title - Artist" format matching channel
        let (artist, title) = parse_track_title_artist("Blinding Lights - The Weeknd", "The Weeknd");
        assert_eq!(artist, "The Weeknd");
        assert_eq!(title, "Blinding Lights");
    }

    #[test]
    fn test_error_categorization_quota_and_key() {
        // Quota error (HTTP 403)
        let quota_json = r#"{
            "error": {
                "code": 403,
                "message": "The request cannot be completed because you have exceeded your quota.",
                "errors": [
                    {
                        "message": "The request cannot be completed because you have exceeded your quota.",
                        "domain": "youtube.quota",
                        "reason": "quotaExceeded"
                    }
                ]
            }
        }"#;
        assert_eq!(parse_youtube_error(403, quota_json), QUOTA_EXCEEDED_ERROR);

        // Daily limit exceeded (HTTP 403)
        let daily_limit_json = r#"{
            "error": {
                "code": 403,
                "message": "Daily Limit Exceeded",
                "errors": [{ "reason": "dailyLimitExceeded" }]
            }
        }"#;
        assert_eq!(parse_youtube_error(403, daily_limit_json), QUOTA_EXCEEDED_ERROR);

        // Invalid API Key (HTTP 400)
        let invalid_key_json = r#"{
            "error": {
                "code": 400,
                "message": "API key not valid. Please pass a valid API key.",
                "errors": [{ "reason": "keyInvalid" }]
            }
        }"#;
        assert_eq!(parse_youtube_error(400, invalid_key_json), KEY_INVALID_ERROR);

        // Service not enabled in GCP project (HTTP 403)
        let not_enabled_json = r#"{
            "error": {
                "code": 403,
                "message": "YouTube Data API v3 has not been used in project 123 before or it is disabled.",
                "errors": [{ "reason": "accessNotConfigured" }]
            }
        }"#;
        assert_eq!(parse_youtube_error(403, not_enabled_json), ACCESS_NOT_CONFIGURED_ERROR);

        // Generic 503 Service Unavailable
        let generic_503 = parse_youtube_error(503, "{}");
        assert!(generic_503.contains("temporarily unavailable"));
    }

    #[test]
    fn test_parse_search_track_json_deserialization() {
        let json_data = r#"{
            "kind": "youtube#searchListResponse",
            "items": [
                {
                    "kind": "youtube#searchResult",
                    "id": {
                        "kind": "youtube#video",
                        "videoId": "kJQP7kiw5Fk"
                    },
                    "snippet": {
                        "title": "Luis Fonsi - Despacito ft. Daddy Yankee",
                        "description": "Despacito official video",
                        "channelTitle": "LuisFonsiVEVO",
                        "thumbnails": {
                            "high": {
                                "url": "https://i.ytimg.com/vi/kJQP7kiw5Fk/hqdefault.jpg",
                                "width": 480,
                                "height": 360
                            }
                        }
                    }
                }
            ]
        }"#;

        let parsed: YouTubeSearchListResponse = serde_json::from_str(json_data).expect("valid search json");
        let items = parsed.items.expect("items present");
        assert_eq!(items.len(), 1);

        let track = parse_search_track_item(&items[0], 288.0).expect("parsed stream track");
        assert_eq!(track.id, "kJQP7kiw5Fk");
        assert_eq!(track.artist, "Luis Fonsi");
        assert_eq!(track.title, "Despacito ft. Daddy Yankee");
        assert_eq!(track.album, "YouTube Music");
        assert_eq!(track.duration_secs, 288.0);
        assert_eq!(
            track.art_url.as_deref(),
            Some("https://i.ytimg.com/vi/kJQP7kiw5Fk/hqdefault.jpg")
        );
        assert_eq!(track.uri, "https://www.youtube.com/watch?v=kJQP7kiw5Fk");
        assert_eq!(track.provider, "youtube");
    }

    #[test]
    fn test_parse_playlist_item_json_deserialization() {
        let json_data = r#"{
            "kind": "youtube#searchResult",
            "id": {
                "kind": "youtube#playlist",
                "playlistId": "PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI"
            },
            "snippet": {
                "title": "Top &amp; Trending Pop Hits",
                "description": "The best pop music hits today.",
                "thumbnails": {
                    "high": {
                        "url": "https://i.ytimg.com/pl/pop.jpg",
                        "width": 480,
                        "height": 360
                    }
                }
            }
        }"#;

        let item: YouTubeSearchResult = serde_json::from_str(json_data).expect("valid playlist item json");
        let playlist = parse_playlist_item(&item).expect("parsed stream playlist");

        assert_eq!(playlist.id, "PLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI");
        assert_eq!(playlist.name, "Top & Trending Pop Hits");
        assert_eq!(
            playlist.description.as_deref(),
            Some("The best pop music hits today.")
        );
        assert_eq!(playlist.art_url.as_deref(), Some("https://i.ytimg.com/pl/pop.jpg"));
    }

    #[test]
    fn test_youtube_provider_traits_and_lifecycle() {
        let mut provider = YouTubeProvider::new().with_api_key("test_api_key_123");

        assert_eq!(provider.id(), "youtube");
        assert_eq!(provider.name(), "YouTube Music");
        assert!(provider.is_connected());

        let track = StreamTrack::new(
            "test_vid_1",
            "Test Song",
            "Test Artist",
            "YouTube Music",
            180.0,
            None,
            "https://www.youtube.com/watch?v=test_vid_1",
            "youtube",
        );

        // Playback lifecycle
        assert!(provider.play(&track).is_ok());

        let state_opt = provider.get_playback_state().expect("get state");
        assert!(state_opt.is_some());
        let state = state_opt.unwrap();
        assert!(state.is_playing);
        assert_eq!(state.current_track.as_ref().unwrap().id, "test_vid_1");
        assert_eq!(state.duration_secs, 180.0);

        // Pause
        assert!(provider.pause().is_ok());
        let paused_state = provider.get_playback_state().unwrap().unwrap();
        assert!(!paused_state.is_playing);

        // Resume
        assert!(provider.resume().is_ok());
        let resumed_state = provider.get_playback_state().unwrap().unwrap();
        assert!(resumed_state.is_playing);

        // Seek
        assert!(provider.seek(60.0).is_ok());
        let seek_state = provider.get_playback_state().unwrap().unwrap();
        assert!(seek_state.position_secs >= 60.0);

        // Volume
        assert!(provider.set_volume(0.75).is_ok());
        let vol_state = provider.get_playback_state().unwrap().unwrap();
        assert!((vol_state.volume - 0.75).abs() < 1e-4);

        // Disconnect
        assert!(provider.disconnect().is_ok());
        assert!(!provider.is_connected());
    }

    #[test]
    fn test_missing_api_key_returns_error() {
        let provider = YouTubeProvider::new();
        // Clear in-memory key explicitly
        *provider.api_key.write().unwrap() = None;

        let res = provider.search("Synthwave");
        assert!(res.is_err());
        assert_eq!(res.err().unwrap(), MISSING_KEY_ERROR);
    }
}
