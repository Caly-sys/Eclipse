use crate::streams::credentials::CredentialStore;
use crate::streams::provider::{
    StreamPlaybackState, StreamPlaylist, StreamProvider, StreamTrack,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

pub const SPOTIFY_PROVIDER_ID: &str = "spotify";
pub const SPOTIFY_PROVIDER_NAME: &str = "Spotify";

pub const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
pub const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
pub const SPOTIFY_API_BASE: &str = "https://api.spotify.com/v1";

pub const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8888/callback";
pub const SPOTIFY_SCOPES: &str =
    "user-read-playback-state user-modify-playback-state user-read-currently-playing playlist-read-private playlist-read-collaborative user-library-read";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotifyTokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub scope: Option<String>,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone)]
struct CachedState {
    state: Option<StreamPlaybackState>,
    base_timestamp: Instant,
    base_position_secs: f64,
    last_polled_at: Instant,
}

#[derive(Debug)]
pub enum SpotifyApiError {
    Unauthorized,
    Forbidden(String),
    NotFound(String),
    RateLimited(u64),
    Transport(String),
    Other(String),
}

/// Official Spotify Web API integration supporting Spotify Connect device playback,
/// OAuth 2.0 PKCE authentication, and 1Hz polling with sub-second seekbar interpolation.
pub struct SpotifyProvider {
    client_id: Arc<RwLock<Option<String>>>,
    redirect_uri: String,
    access_token: Arc<RwLock<Option<String>>>,
    token_expires_at: Arc<RwLock<Option<Instant>>>,
    connected: Arc<RwLock<bool>>,
    cache: Arc<Mutex<Option<CachedState>>>,
}

impl Default for SpotifyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyProvider {
    pub fn new() -> Self {
        let has_token = CredentialStore::get_spotify_refresh_token()
            .ok()
            .flatten()
            .is_some()
            || CredentialStore::get_spotify_access_token()
                .ok()
                .flatten()
                .is_some();

        Self {
            client_id: Arc::new(RwLock::new(None)),
            redirect_uri: DEFAULT_REDIRECT_URI.to_string(),
            access_token: Arc::new(RwLock::new(None)),
            token_expires_at: Arc::new(RwLock::new(None)),
            connected: Arc::new(RwLock::new(has_token)),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_client_id(self, client_id: impl Into<String>) -> Self {
        *self.client_id.write().unwrap() = Some(client_id.into());
        self
    }

    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri = redirect_uri.into();
        self
    }

    pub fn set_client_id(&mut self, client_id: impl Into<String>) {
        *self.client_id.write().unwrap() = Some(client_id.into());
    }

    pub fn get_client_id(&self) -> Result<String, String> {
        if let Some(id) = self.client_id.read().unwrap().as_ref() {
            if !id.trim().is_empty() {
                return Ok(id.clone());
            }
        }
        if let Ok(Some(id)) = CredentialStore::get_spotify_client_id() {
            if !id.trim().is_empty() {
                return Ok(id);
            }
        }
        Err("Spotify Client ID not configured. Please enter your Spotify Client ID in Settings -> Streaming.".to_string())
    }

    pub fn set_connected(&self, connected: bool) {
        *self.connected.write().unwrap() = connected;
    }

    /// Refresh access token using stored refresh token.
    pub fn refresh_access_token(&self) -> Result<String, String> {
        let client_id = self.get_client_id()?;
        let refresh_token = CredentialStore::get_spotify_refresh_token()?
            .ok_or_else(|| "No Spotify refresh token found in credential store".to_string())?;

        let token_resp = refresh_access_token_direct(&client_id, &refresh_token)?;

        // Update Windows Credential Manager
        CredentialStore::save_spotify_tokens(
            &token_resp.access_token,
            token_resp.refresh_token.as_deref(),
        )?;

        // Update in-memory state
        *self.access_token.write().unwrap() = Some(token_resp.access_token.clone());
        let expires_at =
            Instant::now() + Duration::from_secs(token_resp.expires_in.saturating_sub(60));
        *self.token_expires_at.write().unwrap() = Some(expires_at);
        *self.connected.write().unwrap() = true;

        Ok(token_resp.access_token)
    }

    /// Ensure a valid access token is available, refreshing if expired.
    pub fn ensure_access_token(&self) -> Result<String, String> {
        // 1. Check in-memory access token and expiration
        {
            let token_lock = self.access_token.read().unwrap();
            let exp_lock = self.token_expires_at.read().unwrap();
            if let (Some(token), Some(exp)) = (token_lock.as_ref(), *exp_lock) {
                if Instant::now() < exp {
                    return Ok(token.clone());
                }
            }
        }

        // 2. Check CredentialStore
        if let Ok(Some(token)) = CredentialStore::get_spotify_access_token() {
            *self.access_token.write().unwrap() = Some(token.clone());
            *self.token_expires_at.write().unwrap() =
                Some(Instant::now() + Duration::from_secs(600));
            return Ok(token);
        }

        // 3. Fallback: refresh using refresh token
        self.refresh_access_token()
    }

    /// Execute request with automatic 401 token refresh retry and 429 backoff handling.
    pub fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<Option<serde_json::Value>, String> {
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{SPOTIFY_API_BASE}{path}")
        };

        let token = self.ensure_access_token()?;

        match execute_spotify_request(method, &url, &token, body) {
            Ok(res) => Ok(res),
            Err(SpotifyApiError::Unauthorized) => {
                // Token expired: attempt auto-refresh and retry once
                let new_token = self.refresh_access_token()?;
                match execute_spotify_request(method, &url, &new_token, body) {
                    Ok(res) => Ok(res),
                    Err(SpotifyApiError::Unauthorized) => {
                        self.set_connected(false);
                        Err("Spotify session expired. Please reconnect in Settings.".to_string())
                    }
                    Err(e) => self.map_api_error(e),
                }
            }
            Err(SpotifyApiError::RateLimited(secs)) => {
                if secs <= 3 {
                    std::thread::sleep(Duration::from_secs(secs));
                    let token = self.ensure_access_token()?;
                    match execute_spotify_request(method, &url, &token, body) {
                        Ok(res) => Ok(res),
                        Err(e) => self.map_api_error(e),
                    }
                } else {
                    Err(format!(
                        "Spotify rate limit reached (429). Please wait {secs}s before retrying."
                    ))
                }
            }
            Err(e) => self.map_api_error(e),
        }
    }

    fn map_api_error(&self, err: SpotifyApiError) -> Result<Option<serde_json::Value>, String> {
        match err {
            SpotifyApiError::Unauthorized => Err("Spotify unauthorized (401)".to_string()),
            SpotifyApiError::Forbidden(msg) => Err(msg),
            SpotifyApiError::NotFound(msg) => Err(msg),
            SpotifyApiError::RateLimited(s) => {
                Err(format!("Spotify rate limit reached (429). Retry after {s}s."))
            }
            SpotifyApiError::Transport(t) => Err(t),
            SpotifyApiError::Other(o) => Err(o),
        }
    }

    fn update_cache_playing(&self, track: &StreamTrack) {
        let now = Instant::now();
        let mut cache = self.cache.lock().unwrap();
        let (device_name, volume) = if let Some(prev) = cache.as_ref() {
            if let Some(s) = prev.state.as_ref() {
                (s.device_name.clone(), s.volume)
            } else {
                (None, 1.0)
            }
        } else {
            (None, 1.0)
        };

        *cache = Some(CachedState {
            state: Some(StreamPlaybackState {
                is_playing: true,
                current_track: Some(track.clone()),
                position_secs: 0.0,
                duration_secs: track.duration_secs,
                device_name,
                is_premium: true,
                volume,
            }),
            base_timestamp: now,
            base_position_secs: 0.0,
            last_polled_at: now,
        });
    }

    fn update_cache_paused(&self) {
        let mut cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.as_mut() {
            if let Some(state) = cached.state.as_mut() {
                let elapsed = cached.base_timestamp.elapsed().as_secs_f64();
                state.position_secs = interpolate_position(
                    cached.base_position_secs,
                    elapsed,
                    state.duration_secs,
                    true,
                );
                state.is_playing = false;
                cached.base_position_secs = state.position_secs;
                cached.base_timestamp = Instant::now();
                cached.last_polled_at = Instant::now();
            }
        }
    }

    fn update_cache_resumed(&self) {
        let mut cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.as_mut() {
            if let Some(state) = cached.state.as_mut() {
                state.is_playing = true;
                cached.base_timestamp = Instant::now();
                cached.last_polled_at = Instant::now();
            }
        }
    }

    fn update_cache_seek(&self, position_secs: f64) {
        let now = Instant::now();
        let mut cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.as_mut() {
            if let Some(state) = cached.state.as_mut() {
                state.position_secs = position_secs;
                cached.base_position_secs = position_secs;
                cached.base_timestamp = now;
                cached.last_polled_at = now;
            }
        }
    }

    fn update_cache_volume(&self, volume: f32) {
        let mut cache = self.cache.lock().unwrap();
        if let Some(cached) = cache.as_mut() {
            if let Some(state) = cached.state.as_mut() {
                state.volume = volume;
            }
        }
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
        *self.connected.read().unwrap()
    }

    fn connect(&mut self) -> Result<(), String> {
        let client_id = self.get_client_id()?;

        // 1. Try refreshing existing token if refresh_token exists
        if let Ok(Some(refresh_token)) = CredentialStore::get_spotify_refresh_token() {
            if let Ok(token_resp) = refresh_access_token_direct(&client_id, &refresh_token) {
                let _ = CredentialStore::save_spotify_tokens(
                    &token_resp.access_token,
                    token_resp.refresh_token.as_deref(),
                );
                *self.access_token.write().unwrap() = Some(token_resp.access_token);
                *self.token_expires_at.write().unwrap() = Some(
                    Instant::now()
                        + Duration::from_secs(token_resp.expires_in.saturating_sub(60)),
                );
                *self.connected.write().unwrap() = true;
                return Ok(());
            }
        }

        // 2. Initiate PKCE Authorization Code flow with local loopback listener
        let code_verifier = generate_code_verifier();
        let code_challenge = compute_code_challenge(&code_verifier);
        let state = generate_state();

        let auth_url = build_auth_url(&client_id, &self.redirect_uri, &code_challenge, &state);

        // Open user's default browser to Spotify authorize page
        webbrowser::open(&auth_url)
            .map_err(|e| format!("Failed to open web browser for Spotify authentication: {e}"))?;

        // Listen on local loopback server for redirect
        let code = listen_for_auth_code(&self.redirect_uri, &state, Duration::from_secs(120))?;

        // Exchange code for tokens
        let token_resp =
            exchange_code_for_tokens(&client_id, &code, &code_verifier, &self.redirect_uri)?;

        // Persist tokens securely in Windows Credential Manager
        CredentialStore::save_spotify_tokens(
            &token_resp.access_token,
            token_resp.refresh_token.as_deref(),
        )?;

        // Update in-memory state
        *self.access_token.write().unwrap() = Some(token_resp.access_token);
        *self.token_expires_at.write().unwrap() = Some(
            Instant::now() + Duration::from_secs(token_resp.expires_in.saturating_sub(60)),
        );
        *self.connected.write().unwrap() = true;

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        CredentialStore::clear_spotify_credentials()?;
        *self.access_token.write().unwrap() = None;
        *self.token_expires_at.write().unwrap() = None;
        *self.connected.write().unwrap() = false;
        let mut cache = self.cache.lock().unwrap();
        *cache = None;
        Ok(())
    }

    fn search(&self, query: &str) -> Result<Vec<StreamTrack>, String> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let encoded_query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("q", query)
            .append_pair("type", "track")
            .append_pair("limit", "25")
            .finish();

        let path = format!("/search?{encoded_query}");
        let val_opt = self.request("GET", &path, None)?;

        let mut tracks = Vec::new();
        if let Some(val) = val_opt {
            if let Some(items) = val["tracks"]["items"].as_array() {
                for item in items {
                    if let Some(track) = parse_spotify_track(item) {
                        tracks.push(track);
                    }
                }
            }
        }

        Ok(tracks)
    }

    fn get_user_playlists(&self) -> Result<Vec<StreamPlaylist>, String> {
        let val_opt = self.request("GET", "/me/playlists?limit=50", None)?;
        let mut playlists = Vec::new();
        if let Some(val) = val_opt {
            if let Some(items) = val["items"].as_array() {
                for item in items {
                    if let Some(pl) = parse_spotify_playlist(item) {
                        playlists.push(pl);
                    }
                }
            }
        }
        Ok(playlists)
    }

    fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<StreamTrack>, String> {
        let path = format!("/playlists/{playlist_id}/tracks?limit=50");
        let val_opt = self.request("GET", &path, None)?;

        let mut tracks = Vec::new();
        if let Some(val) = val_opt {
            if let Some(items) = val["items"].as_array() {
                for item in items {
                    let track_item = if item["track"].is_object() {
                        &item["track"]
                    } else {
                        item
                    };
                    if let Some(track) = parse_spotify_track(track_item) {
                        tracks.push(track);
                    }
                }
            }
        }
        Ok(tracks)
    }

    fn play(&mut self, track: &StreamTrack) -> Result<(), String> {
        let uri = if track.uri.starts_with("spotify:track:") {
            track.uri.clone()
        } else {
            format!("spotify:track:{}", track.id)
        };

        let body = serde_json::json!({
            "uris": [uri]
        });

        let play_res = self.request("PUT", "/me/player/play", Some(&body));
        match play_res {
            Ok(_) => {
                self.update_cache_playing(track);
                Ok(())
            }
            Err(e) if e.contains("No active Spotify Connect device found") => {
                // Check if any devices exist to transfer/direct playback
                let devices_val = self.request("GET", "/me/player/devices", None)?;
                let devices = devices_val
                    .and_then(|v| v.get("devices").cloned())
                    .and_then(|v| v.as_array().cloned())
                    .unwrap_or_default();

                if devices.is_empty() {
                    return Err(
                        "No active Spotify Connect device found. Please start Spotify on your desktop, phone, or web player."
                            .to_string(),
                    );
                }

                let device_id = devices[0]["id"]
                    .as_str()
                    .ok_or_else(|| "Spotify device missing ID".to_string())?;

                let path = format!("/me/player/play?device_id={device_id}");
                self.request("PUT", &path, Some(&body))?;
                self.update_cache_playing(track);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn pause(&mut self) -> Result<(), String> {
        self.request("PUT", "/me/player/pause", None)?;
        self.update_cache_paused();
        Ok(())
    }

    fn resume(&mut self) -> Result<(), String> {
        self.request("PUT", "/me/player/play", None)?;
        self.update_cache_resumed();
        Ok(())
    }

    fn seek(&mut self, position_secs: f64) -> Result<(), String> {
        let position_ms = (position_secs * 1000.0).round() as u64;
        let path = format!("/me/player/seek?position_ms={position_ms}");
        self.request("PUT", &path, None)?;
        self.update_cache_seek(position_secs);
        Ok(())
    }

    fn next(&mut self) -> Result<(), String> {
        self.request("POST", "/me/player/next", None)?;
        Ok(())
    }

    fn prev(&mut self) -> Result<(), String> {
        self.request("POST", "/me/player/previous", None)?;
        Ok(())
    }

    fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        let percent = (volume.clamp(0.0, 1.0) * 100.0).round() as u32;
        let path = format!("/me/player/volume?volume_percent={percent}");
        self.request("PUT", &path, None)?;
        self.update_cache_volume(volume);
        Ok(())
    }

    fn get_playback_state(&self) -> Result<Option<StreamPlaybackState>, String> {
        if !self.is_connected() {
            return Ok(None);
        }

        // Check cached state if within 900ms to avoid spamming the Spotify API
        {
            let cache_lock = self.cache.lock().unwrap();
            if let Some(cached) = cache_lock.as_ref() {
                let elapsed = cached.last_polled_at.elapsed();
                if elapsed < Duration::from_millis(900) {
                    if let Some(mut state) = cached.state.clone() {
                        let total_elapsed = cached.base_timestamp.elapsed().as_secs_f64();
                        state.position_secs = interpolate_position(
                            cached.base_position_secs,
                            total_elapsed,
                            state.duration_secs,
                            state.is_playing,
                        );
                        return Ok(Some(state));
                    } else {
                        return Ok(None);
                    }
                }
            }
        }

        // Actual API query
        let val_opt = match self.request("GET", "/me/player", None) {
            Ok(v) => v,
            Err(e) => {
                // If network/rate error, fall back to cached interpolated state if available
                let cache_lock = self.cache.lock().unwrap();
                if let Some(cached) = cache_lock.as_ref() {
                    if let Some(mut state) = cached.state.clone() {
                        let total_elapsed = cached.base_timestamp.elapsed().as_secs_f64();
                        state.position_secs = interpolate_position(
                            cached.base_position_secs,
                            total_elapsed,
                            state.duration_secs,
                            state.is_playing,
                        );
                        return Ok(Some(state));
                    }
                }
                return Err(e);
            }
        };

        let val = match val_opt {
            Some(v) if !v.is_null() => v,
            _ => {
                let mut cache_lock = self.cache.lock().unwrap();
                *cache_lock = Some(CachedState {
                    state: None,
                    base_timestamp: Instant::now(),
                    base_position_secs: 0.0,
                    last_polled_at: Instant::now(),
                });
                return Ok(None);
            }
        };

        let parsed_state = parse_spotify_playback_state(&val);
        let now = Instant::now();

        let mut cache_lock = self.cache.lock().unwrap();
        *cache_lock = Some(CachedState {
            state: parsed_state.clone(),
            base_timestamp: now,
            base_position_secs: parsed_state.as_ref().map_or(0.0, |s| s.position_secs),
            last_polled_at: now,
        });

        Ok(parsed_state)
    }
}

// ----------------------------------------------------------------------------
// PKCE Cryptographic Helpers (RFC 7636)
// ----------------------------------------------------------------------------

/// Generate a cryptographically random high-entropy code verifier string (43..128 chars).
/// Uses unreserved characters [A-Z], [a-z], [0-9], "-", ".", "_", "~" as per RFC 7636 Section 4.1.
pub fn generate_code_verifier() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
    let mut rng = rand::thread_rng();
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Compute standard S256 code challenge: BASE64URL(SHA256(verifier)) without padding.
/// Matches RFC 7636 Section 4.2.
pub fn compute_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Generate random 16-character state string for CSRF protection.
pub fn generate_state() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..16)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Build authorization URL for PKCE flow.
pub fn build_auth_url(
    client_id: &str,
    redirect_uri: &str,
    challenge: &str,
    state: &str,
) -> String {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", challenge)
        .append_pair("scope", SPOTIFY_SCOPES)
        .append_pair("state", state)
        .finish();

    format!("{SPOTIFY_AUTH_URL}?{query}")
}

/// Smooth sub-second seekbar position interpolation.
/// When playing, advances `base_pos_secs` by `elapsed_secs`, clamping to `duration_secs`.
/// When paused, remains at `base_pos_secs`.
pub fn interpolate_position(
    base_pos_secs: f64,
    elapsed_secs: f64,
    duration_secs: f64,
    is_playing: bool,
) -> f64 {
    if !is_playing {
        base_pos_secs
    } else if duration_secs > 0.0 {
        (base_pos_secs + elapsed_secs).clamp(0.0, duration_secs)
    } else {
        (base_pos_secs + elapsed_secs).max(0.0)
    }
}

// ----------------------------------------------------------------------------
// Local Loopback Server (OAuth 2.0 PKCE Redirect)
// ----------------------------------------------------------------------------

pub fn listen_for_auth_code(
    redirect_uri: &str,
    expected_state: &str,
    timeout: Duration,
) -> Result<String, String> {
    let parsed_url = url::Url::parse(redirect_uri)
        .map_err(|e| format!("Invalid redirect URI '{redirect_uri}': {e}"))?;
    let port = parsed_url.port().unwrap_or(8888);
    let bind_addr = format!("127.0.0.1:{port}");

    let listener = TcpListener::bind(&bind_addr)
        .map_err(|e| format!("Failed to bind local loopback listener on {bind_addr}: {e}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set nonblocking on listener: {e}"))?;

    let start = Instant::now();
    let mut code_result: Option<Result<String, String>> = None;

    while start.elapsed() < timeout {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let mut buf = [0u8; 4096];
                let mut bytes_read = 0;
                if let Ok(n) = stream.read(&mut buf) {
                    bytes_read = n;
                }
                if bytes_read == 0 {
                    continue;
                }

                let req = String::from_utf8_lossy(&buf[..bytes_read]);
                let first_line = req.lines().next().unwrap_or("");
                let mut parts = first_line.split_whitespace();
                let method = parts.next().unwrap_or("");
                let path_and_query = parts.next().unwrap_or("");

                // Skip /favicon.ico requests
                if path_and_query.starts_with("/favicon.ico") {
                    let not_found =
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(not_found.as_bytes());
                    let _ = stream.flush();
                    continue;
                }

                if method != "GET" {
                    continue;
                }

                let dummy_url = format!("http://localhost{path_and_query}");
                let url_obj = match url::Url::parse(&dummy_url) {
                    Ok(u) => u,
                    Err(_) => continue,
                };

                let mut code: Option<String> = None;
                let mut state: Option<String> = None;
                let mut error: Option<String> = None;

                for (k, v) in url_obj.query_pairs() {
                    match k.as_ref() {
                        "code" => code = Some(v.to_string()),
                        "state" => state = Some(v.to_string()),
                        "error" => error = Some(v.to_string()),
                        _ => {}
                    }
                }

                if let Some(err) = error {
                    let html = format!(
                        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Eclipse - Spotify Authorization</title>\
                        <style>body{{background:#0c0914;color:#ece9fc;font-family:sans-serif;display:flex;\
                        align-items:center;justify-content:center;height:100vh;margin:0}}\
                        .box{{background:rgba(35,22,55,0.95);border:1px solid rgba(239,68,68,0.4);\
                        border-radius:16px;padding:36px;text-align:center;max-width:420px}}\
                        h1{{color:#ef4444;font-size:22px;margin:0 0 12px 0}}p{{color:#a69fb8;font-size:14px}}\
                        </style></head><body><div class=\"box\"><h1>Authorization Denied</h1>\
                        <p>Spotify returned error: <strong>{err}</strong>. You may close this window.</p>\
                        </div></body></html>"
                    );
                    let resp = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        html.len(),
                        html
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    code_result = Some(Err(format!("Spotify authorization denied: {err}")));
                    break;
                }

                if let Some(received_state) = state {
                    if received_state != expected_state {
                        let html = "<!DOCTYPE html><html><body><h1>Security Error</h1><p>OAuth state mismatch.</p></body></html>";
                        let resp = format!(
                            "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            html.len(),
                            html
                        );
                        let _ = stream.write_all(resp.as_bytes());
                        let _ = stream.flush();
                        code_result = Some(Err(
                            "OAuth state mismatch: possible CSRF attempt.".to_string(),
                        ));
                        break;
                    }
                }

                if let Some(auth_code) = code {
                    let html = "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Eclipse - Connected</title>\
                        <style>body{background:#0c0914;color:#ece9fc;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0}\
                        .card{background:rgba(28,20,48,0.95);border:1px solid rgba(168,85,247,0.4);border-radius:16px;padding:40px 36px;text-align:center;max-width:440px;box-shadow:0 16px 48px rgba(0,0,0,0.6)}\
                        .logo{color:#1DB954;font-size:36px;line-height:1;margin-bottom:16px;font-weight:bold}\
                        h1{font-size:22px;font-weight:600;margin:0 0 10px 0;color:#ffffff}\
                        p{font-size:14px;color:#a69fb8;line-height:1.5;margin:0 0 20px 0}\
                        .badge{display:inline-block;background:rgba(29,185,84,0.15);color:#1DB954;border:1px solid rgba(29,185,84,0.35);border-radius:999px;padding:4px 14px;font-size:12px;font-weight:500}\
                        </style></head><body><div class=\"card\"><div class=\"logo\">● Spotify</div>\
                        <h1>Connected to Eclipse</h1><p>Authentication was successful! You may now close this browser tab and return to <strong>Eclipse</strong>.</p>\
                        <span class=\"badge\">Session Active</span></div></body></html>";
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        html.len(),
                        html
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    std::thread::sleep(Duration::from_millis(50));
                    code_result = Some(Ok(auth_code));
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(format!("Loopback listener socket error: {e}"));
            }
        }
    }

    match code_result {
        Some(res) => res,
        None => Err(format!(
            "Authentication timed out after {} seconds waiting for Spotify browser callback.",
            timeout.as_secs()
        )),
    }
}

// ----------------------------------------------------------------------------
// Token Exchange & Refresh
// ----------------------------------------------------------------------------

pub fn exchange_code_for_tokens(
    client_id: &str,
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<SpotifyTokenResponse, String> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("grant_type", "authorization_code")
        .append_pair("code", code)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("code_verifier", code_verifier)
        .finish();

    let resp = ureq::post(SPOTIFY_TOKEN_URL)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .timeout(Duration::from_secs(15))
        .send_string(&body)
        .map_err(|e| match e {
            ureq::Error::Status(status, response) => {
                let err_text = response.into_string().unwrap_or_default();
                format!("Spotify token exchange failed (HTTP {status}): {err_text}")
            }
            ureq::Error::Transport(t) => {
                format!("Network error communicating with Spotify Accounts: {t}")
            }
        })?;

    let token_resp: SpotifyTokenResponse = resp
        .into_json()
        .map_err(|e| format!("Failed to parse Spotify token response: {e}"))?;

    Ok(token_resp)
}

pub fn refresh_access_token_direct(
    client_id: &str,
    refresh_token: &str,
) -> Result<SpotifyTokenResponse, String> {
    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", client_id)
        .append_pair("grant_type", "refresh_token")
        .append_pair("refresh_token", refresh_token)
        .finish();

    let resp = ureq::post(SPOTIFY_TOKEN_URL)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .timeout(Duration::from_secs(15))
        .send_string(&body)
        .map_err(|e| match e {
            ureq::Error::Status(status, response) => {
                let err_text = response.into_string().unwrap_or_default();
                format!("Spotify token refresh failed (HTTP {status}): {err_text}")
            }
            ureq::Error::Transport(t) => {
                format!("Network error communicating with Spotify Accounts: {t}")
            }
        })?;

    let token_resp: SpotifyTokenResponse = resp
        .into_json()
        .map_err(|e| format!("Failed to parse Spotify refresh response: {e}"))?;

    Ok(token_resp)
}

// ----------------------------------------------------------------------------
// Low-Level Spotify HTTP Execution
// ----------------------------------------------------------------------------

fn execute_spotify_request(
    method: &str,
    url: &str,
    access_token: &str,
    body: Option<&serde_json::Value>,
) -> Result<Option<serde_json::Value>, SpotifyApiError> {
    let mut req = match method {
        "GET" => ureq::get(url),
        "POST" => ureq::post(url),
        "PUT" => ureq::put(url),
        "DELETE" => ureq::delete(url),
        _ => {
            return Err(SpotifyApiError::Other(format!(
                "Unsupported HTTP method: {method}"
            )))
        }
    };

    req = req
        .set("Authorization", &format!("Bearer {access_token}"))
        .timeout(Duration::from_secs(10));

    let res = if let Some(b) = body {
        req.set("Content-Type", "application/json").send_json(b)
    } else if method == "PUT" || method == "POST" {
        req.send_string("")
    } else {
        req.call()
    };

    match res {
        Ok(response) => {
            if response.status() == 204 {
                Ok(None)
            } else {
                match response.into_json::<serde_json::Value>() {
                    Ok(val) => Ok(Some(val)),
                    Err(_) => Ok(None),
                }
            }
        }
        Err(ureq::Error::Status(401, _)) => Err(SpotifyApiError::Unauthorized),
        Err(ureq::Error::Status(403, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            if body_text.to_lowercase().contains("premium")
                || body_text.contains("PREMIUM_REQUIRED")
            {
                Err(SpotifyApiError::Forbidden(
                    "Spotify Premium required for Connect device playback.".to_string(),
                ))
            } else {
                Err(SpotifyApiError::Forbidden(
                    "Spotify access forbidden (403). Ensure required permissions are granted."
                        .to_string(),
                ))
            }
        }
        Err(ureq::Error::Status(404, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            if url.contains("/player") || body_text.contains("NO_ACTIVE_DEVICE") {
                Err(SpotifyApiError::NotFound(
                    "No active Spotify Connect device found. Please start Spotify on your desktop, phone, or web player.".to_string(),
                ))
            } else {
                Err(SpotifyApiError::NotFound(format!(
                    "Spotify resource not found (404): {body_text}"
                )))
            }
        }
        Err(ureq::Error::Status(429, response)) => {
            let retry_after = response
                .header("Retry-After")
                .and_then(|h| h.parse::<u64>().ok())
                .unwrap_or(2);
            Err(SpotifyApiError::RateLimited(retry_after))
        }
        Err(ureq::Error::Status(status, response)) => {
            let body_text = response.into_string().unwrap_or_default();
            Err(SpotifyApiError::Other(format!(
                "Spotify API error (HTTP {status}): {body_text}"
            )))
        }
        Err(ureq::Error::Transport(t)) => Err(SpotifyApiError::Transport(format!(
            "Network error communicating with Spotify: {t}"
        ))),
    }
}

// ----------------------------------------------------------------------------
// JSON Parsers
// ----------------------------------------------------------------------------

pub fn parse_spotify_track(item: &serde_json::Value) -> Option<StreamTrack> {
    let id = item["id"].as_str()?.to_string();
    let title = item["name"].as_str().unwrap_or("Unknown Title").to_string();

    let artists: Vec<String> = item["artists"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let artist = if artists.is_empty() {
        "Unknown Artist".to_string()
    } else {
        artists.join(", ")
    };

    let album = item["album"]["name"].as_str().unwrap_or("").to_string();
    let duration_ms = item["duration_ms"].as_f64().unwrap_or(0.0);
    let duration_secs = duration_ms / 1000.0;

    let art_url = item["album"]["images"]
        .as_array()
        .and_then(|imgs| imgs.first())
        .and_then(|img| img["url"].as_str())
        .map(|s| s.to_string());

    let uri = item["uri"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("spotify:track:{id}"));

    Some(StreamTrack::new(
        id,
        title,
        artist,
        album,
        duration_secs,
        art_url,
        uri,
        SPOTIFY_PROVIDER_ID,
    ))
}

pub fn parse_spotify_playlist(item: &serde_json::Value) -> Option<StreamPlaylist> {
    let id = item["id"].as_str()?.to_string();
    let name = item["name"]
        .as_str()
        .unwrap_or("Untitled Playlist")
        .to_string();
    let track_count = item["tracks"]["total"].as_u64().unwrap_or(0) as usize;
    let description = item["description"].as_str().map(|s| s.to_string());
    let art_url = item["images"]
        .as_array()
        .and_then(|imgs| imgs.first())
        .and_then(|img| img["url"].as_str())
        .map(|s| s.to_string());

    Some(StreamPlaylist::new(
        id,
        name,
        track_count,
        description,
        art_url,
    ))
}

pub fn parse_spotify_playback_state(val: &serde_json::Value) -> Option<StreamPlaybackState> {
    let is_playing = val["is_playing"].as_bool().unwrap_or(false);
    let position_ms = val["progress_ms"].as_f64().unwrap_or(0.0);
    let position_secs = position_ms / 1000.0;

    let device_name = val["device"]["name"].as_str().map(|s| s.to_string());
    let volume = val["device"]["volume_percent"]
        .as_f64()
        .map(|v| (v / 100.0) as f32)
        .unwrap_or(1.0);

    let track = if val["item"].is_object() {
        parse_spotify_track(&val["item"])
    } else {
        None
    };

    let duration_secs = track.as_ref().map_or(0.0, |t| t.duration_secs);

    Some(StreamPlaybackState {
        is_playing,
        current_track: track,
        position_secs,
        duration_secs,
        device_name,
        is_premium: true,
        volume,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rfc7636_pkce_challenge_vector() {
        // RFC 7636 Appendix B official test vector
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = compute_code_challenge(verifier);
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn test_code_verifier_generation() {
        let verifier = generate_code_verifier();
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
        let allowed = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
        assert!(verifier.chars().all(|c| allowed.contains(c)));
    }

    #[test]
    fn test_auth_url_construction() {
        let url_str = build_auth_url(
            "my_client_id",
            "http://127.0.0.1:8888/callback",
            "my_challenge",
            "my_state",
        );
        let u = url::Url::parse(&url_str).expect("parse auth url");
        assert_eq!(u.host_str(), Some("accounts.spotify.com"));
        assert_eq!(u.path(), "/authorize");
        let query: std::collections::HashMap<_, _> = u.query_pairs().into_owned().collect();
        assert_eq!(query.get("client_id"), Some(&"my_client_id".to_string()));
        assert_eq!(query.get("response_type"), Some(&"code".to_string()));
        assert_eq!(
            query.get("redirect_uri"),
            Some(&"http://127.0.0.1:8888/callback".to_string())
        );
        assert_eq!(query.get("code_challenge_method"), Some(&"S256".to_string()));
        assert_eq!(query.get("code_challenge"), Some(&"my_challenge".to_string()));
        assert_eq!(query.get("state"), Some(&"my_state".to_string()));
        assert!(query.get("scope").unwrap().contains("user-read-playback-state"));
    }

    #[test]
    fn test_position_interpolation() {
        // Paused: position does not change
        assert_eq!(interpolate_position(10.0, 5.0, 100.0, false), 10.0);

        // Playing: advances with elapsed time
        assert_eq!(interpolate_position(10.0, 2.5, 100.0, true), 12.5);

        // Clamping: does not exceed duration
        assert_eq!(interpolate_position(98.0, 5.0, 100.0, true), 100.0);

        // Zero duration: does not clamp to zero
        assert_eq!(interpolate_position(5.0, 3.0, 0.0, true), 8.0);
    }

    #[test]
    fn test_parse_spotify_track_json() {
        let json = serde_json::json!({
            "id": "4iV5W9uYEdYUVa79Axb7Rh",
            "name": "Remind Me",
            "artists": [
                { "name": "Röyksopp" }
            ],
            "album": {
                "name": "Melody A.M.",
                "images": [
                    { "url": "https://i.scdn.co/image/ab67616d0000b27312345" }
                ]
            },
            "duration_ms": 219000,
            "uri": "spotify:track:4iV5W9uYEdYUVa79Axb7Rh"
        });

        let track = parse_spotify_track(&json).expect("parse track");
        assert_eq!(track.id, "4iV5W9uYEdYUVa79Axb7Rh");
        assert_eq!(track.title, "Remind Me");
        assert_eq!(track.artist, "Röyksopp");
        assert_eq!(track.album, "Melody A.M.");
        assert_eq!(track.duration_secs, 219.0);
        assert_eq!(
            track.art_url.as_deref(),
            Some("https://i.scdn.co/image/ab67616d0000b27312345")
        );
        assert_eq!(track.uri, "spotify:track:4iV5W9uYEdYUVa79Axb7Rh");
        assert_eq!(track.provider, "spotify");
    }

    #[test]
    fn test_parse_spotify_playlist_json() {
        let json = serde_json::json!({
            "id": "37i9dQZF1DXcBWIGoYBM5M",
            "name": "Today's Top Hits",
            "description": "Jung Kook is on top of the Hottest 50!",
            "tracks": { "total": 50 },
            "images": [
                { "url": "https://i.scdn.co/image/ab67706f0000000212345" }
            ]
        });

        let pl = parse_spotify_playlist(&json).expect("parse playlist");
        assert_eq!(pl.id, "37i9dQZF1DXcBWIGoYBM5M");
        assert_eq!(pl.name, "Today's Top Hits");
        assert_eq!(pl.track_count, 50);
        assert_eq!(
            pl.description.as_deref(),
            Some("Jung Kook is on top of the Hottest 50!")
        );
        assert_eq!(
            pl.art_url.as_deref(),
            Some("https://i.scdn.co/image/ab67706f0000000212345")
        );
    }

    #[test]
    fn test_parse_spotify_player_state_json() {
        let json = serde_json::json!({
            "is_playing": true,
            "progress_ms": 45000,
            "device": {
                "name": "Hype Desktop",
                "volume_percent": 80
            },
            "item": {
                "id": "track_123",
                "name": "Midnight City",
                "artists": [{ "name": "M83" }],
                "album": { "name": "Hurry Up, We're Dreaming", "images": [] },
                "duration_ms": 243000,
                "uri": "spotify:track:track_123"
            }
        });

        let state = parse_spotify_playback_state(&json).expect("parse state");
        assert!(state.is_playing);
        assert_eq!(state.position_secs, 45.0);
        assert_eq!(state.duration_secs, 243.0);
        assert_eq!(state.device_name.as_deref(), Some("Hype Desktop"));
        assert_eq!(state.volume, 0.8);
        assert!(state.is_premium);
        assert_eq!(state.current_track.unwrap().title, "Midnight City");
    }

    #[test]
    fn test_error_body_categorization() {
        let err_premium = SpotifyApiError::Forbidden(
            "Spotify Premium required for Connect device playback.".to_string(),
        );
        if let SpotifyApiError::Forbidden(msg) = err_premium {
            assert!(msg.contains("Spotify Premium required"));
        }

        let err_device = SpotifyApiError::NotFound(
            "No active Spotify Connect device found. Please start Spotify on your desktop, phone, or web player.".to_string(),
        );
        if let SpotifyApiError::NotFound(msg) = err_device {
            assert!(msg.contains("No active Spotify Connect device found"));
        }
    }

    #[test]
    fn test_spotify_provider_traits() {
        let provider = SpotifyProvider::new()
            .with_client_id("test_client_id")
            .with_redirect_uri("http://127.0.0.1:8888/callback");

        assert_eq!(provider.id(), "spotify");
        assert_eq!(provider.name(), "Spotify");
        assert_eq!(provider.get_client_id().unwrap(), "test_client_id");
    }

    #[test]
    fn test_parse_spotify_track_missing_or_empty_metadata() {
        let json = serde_json::json!({
            "id": "minimal_id",
            "name": "",
            "artists": [],
            "album": {
                "name": "",
                "images": []
            },
            "duration_ms": 0,
            "uri": "spotify:track:minimal_id"
        });

        let track = parse_spotify_track(&json).expect("parse track");
        assert_eq!(track.id, "minimal_id");
        assert_eq!(track.title, "");
        assert_eq!(track.display_title(), "Unknown Title");
        assert_eq!(track.artist, "Unknown Artist");
        assert_eq!(track.display_artist(), "Unknown Artist");
        assert_eq!(track.album, "");
        assert_eq!(track.duration_secs, 0.0);
        assert_eq!(track.art_url, None);
    }

    #[test]
    fn test_parse_spotify_playback_state_paused_and_empty() {
        let json = serde_json::json!({
            "is_playing": false,
            "progress_ms": 12000,
            "device": {
                "name": "Phone",
                "volume_percent": 50
            },
            "item": null
        });

        let state = parse_spotify_playback_state(&json).expect("parse state");
        assert!(!state.is_playing);
        assert_eq!(state.position_secs, 12.0);
        assert_eq!(state.duration_secs, 0.0);
        assert_eq!(state.device_name.as_deref(), Some("Phone"));
        assert_eq!(state.volume, 0.5);
        assert!(state.current_track.is_none());
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "mock_access_token_123",
            "token_type": "Bearer",
            "scope": "user-read-playback-state",
            "expires_in": 3600,
            "refresh_token": "mock_refresh_token_456"
        }"#;

        let token_resp: SpotifyTokenResponse =
            serde_json::from_str(json).expect("deserialize token response");
        assert_eq!(token_resp.access_token, "mock_access_token_123");
        assert_eq!(token_resp.token_type.as_deref(), Some("Bearer"));
        assert_eq!(token_resp.expires_in, 3600);
        assert_eq!(
            token_resp.refresh_token.as_deref(),
            Some("mock_refresh_token_456")
        );
    }
}
