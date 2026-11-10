//! Zero-login public Spotify metadata importer for Eclipse.
//!
//! Fetches track, album, and playlist metadata from public Spotify links
//! (open.spotify.com URLs and spotify: URIs) via public embed pages and official oEmbed.
//! Requires ZERO login, ZERO client ID, and ZERO developer app configuration.

use crate::library::db::Database;
use crate::streams::provider::StreamTrack;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Supported Spotify resource types for public link importing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpotifyResourceType {
    Track,
    Album,
    Playlist,
}

impl SpotifyResourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SpotifyResourceType::Track => "track",
            SpotifyResourceType::Album => "album",
            SpotifyResourceType::Playlist => "playlist",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SpotifyResourceType::Track => "Track",
            SpotifyResourceType::Album => "Album",
            SpotifyResourceType::Playlist => "Playlist",
        }
    }
}

/// A parsed and validated Spotify resource identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSpotifyTarget {
    pub resource_type: SpotifyResourceType,
    pub id: String,
}

impl ParsedSpotifyTarget {
    /// Return the canonical Spotify URI (e.g. `spotify:track:4cOdK2wGLETKBW3PvgPWqT`).
    pub fn canonical_uri(&self) -> String {
        format!("spotify:{}:{}", self.resource_type.as_str(), self.id)
    }

    /// Return the public embed URL (e.g. `https://open.spotify.com/embed/track/4cOdK2wGLETKBW3PvgPWqT`).
    pub fn embed_url(&self) -> String {
        format!(
            "https://open.spotify.com/embed/{}/{}",
            self.resource_type.as_str(),
            self.id
        )
    }

    /// Return the canonical public open.spotify.com URL.
    pub fn open_url(&self) -> String {
        format!(
            "https://open.spotify.com/{}/{}",
            self.resource_type.as_str(),
            self.id
        )
    }
}

/// Unified result of a public Spotify import operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotifyImportResult {
    pub resource_type: SpotifyResourceType,
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_url: Option<String>,
    pub tracks: Vec<StreamTrack>,
    pub total_tracks: usize,
    pub is_truncated: bool,
    pub truncation_message: Option<String>,
    pub original_url: String,
}

/// Contract for fetching public Spotify metadata without login or API keys.
pub trait SpotifyPublicMetadataFetcher: Send + Sync {
    fn fetch_metadata(&self, input: &str) -> Result<SpotifyImportResult, String>;
}

/// Parse any Spotify URL or URI into a validated `ParsedSpotifyTarget`.
///
/// Supported formats:
/// - `https://open.spotify.com/track/{id}`
/// - `https://open.spotify.com/album/{id}`
/// - `https://open.spotify.com/playlist/{id}`
/// - `https://open.spotify.com/embed/{track|album|playlist}/{id}`
/// - `https://open.spotify.com/intl-{lang}/{track|album|playlist}/{id}`
/// - `https://open.spotify.com/user/{user}/playlist/{id}`
/// - `spotify:track:{id}`, `spotify:album:{id}`, `spotify:playlist:{id}`
/// - `spotify:user:{user}:playlist:{id}`
/// - Preserves ID through query parameters (e.g. `?si=...`) and fragments.
pub fn parse_spotify_link(input: &str) -> Result<ParsedSpotifyTarget, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err("Spotify link or URI cannot be empty.".to_string());
    }

    // 1. Handle spotify: URIs
    if raw.starts_with("spotify:") {
        let parts: Vec<&str> = raw.split(':').collect();
        if parts.len() < 3 {
            return Err(format!(
                "Invalid Spotify URI '{raw}'. Expected format: 'spotify:track:ID', 'spotify:album:ID', or 'spotify:playlist:ID'."
            ));
        }

        // Handle legacy user playlist URIs: spotify:user:{username}:playlist:{id}
        if parts.len() >= 5 && parts[1] == "user" && parts[3] == "playlist" {
            let id = clean_spotify_id(parts[4]);
            validate_spotify_id(&id)?;
            return Ok(ParsedSpotifyTarget {
                resource_type: SpotifyResourceType::Playlist,
                id,
            });
        }

        let kind = parts[1].to_lowercase();
        let res_type = match kind.as_str() {
            "track" => SpotifyResourceType::Track,
            "album" => SpotifyResourceType::Album,
            "playlist" => SpotifyResourceType::Playlist,
            "artist" => {
                return Err(
                    "Spotify artists cannot be imported directly as a playable stream. Please provide a track, album, or playlist link."
                        .to_string(),
                );
            }
            other => {
                return Err(format!(
                    "Unsupported Spotify resource type '{other}'. Eclipse supports tracks, albums, and playlists."
                ));
            }
        };

        let id = clean_spotify_id(parts[2]);
        validate_spotify_id(&id)?;

        return Ok(ParsedSpotifyTarget {
            resource_type: res_type,
            id,
        });
    }

    // 2. Handle HTTP/HTTPS URLs (or bare host URLs like open.spotify.com/...)
    let url_str = if !raw.starts_with("http://") && !raw.starts_with("https://") {
        format!("https://{raw}")
    } else {
        raw.to_string()
    };

    let parsed_url = Url::parse(&url_str).map_err(|e| {
        format!("Failed to parse URL '{raw}': {e}. Please check the link and try again.")
    })?;

    let host = parsed_url.host_str().unwrap_or("").to_lowercase();
    if !host.ends_with("spotify.com") && host != "spotify.link" {
        return Err(format!(
            "The provided link '{raw}' is not a Spotify link. Expected open.spotify.com."
        ));
    }

    // Examine path segments
    let segments: Vec<&str> = parsed_url
        .path_segments()
        .map(|s| s.filter(|seg| !seg.is_empty()).collect())
        .unwrap_or_default();

    if segments.is_empty() {
        return Err(format!(
            "Missing Spotify resource in link '{raw}'. Please provide a track, album, or playlist link."
        ));
    }

    // Look for track / album / playlist in segments
    let mut found_type: Option<SpotifyResourceType> = None;
    let mut id_idx: Option<usize> = None;

    for (i, &seg) in segments.iter().enumerate() {
        let seg_lower = seg.to_lowercase();
        match seg_lower.as_str() {
            "track" => {
                found_type = Some(SpotifyResourceType::Track);
                id_idx = Some(i + 1);
                break;
            }
            "album" => {
                found_type = Some(SpotifyResourceType::Album);
                id_idx = Some(i + 1);
                break;
            }
            "playlist" => {
                found_type = Some(SpotifyResourceType::Playlist);
                id_idx = Some(i + 1);
                break;
            }
            "artist" => {
                return Err(
                    "Spotify artists cannot be imported directly as a playable stream. Please provide a track, album, or playlist link."
                        .to_string(),
                );
            }
            _ => {}
        }
    }

    if let (Some(resource_type), Some(idx)) = (found_type, id_idx) {
        if idx < segments.len() {
            let id = clean_spotify_id(segments[idx]);
            validate_spotify_id(&id)?;
            return Ok(ParsedSpotifyTarget { resource_type, id });
        }
    }

    Err(format!(
        "Could not detect a valid track, album, or playlist in Spotify link '{raw}'. Please check the URL."
    ))
}

/// Strip any query params or fragments if accidentally bundled in the ID segment.
fn clean_spotify_id(id_raw: &str) -> String {
    let mut id = id_raw.trim();
    if let Some(pos) = id.find('?') {
        id = &id[..pos];
    }
    if let Some(pos) = id.find('#') {
        id = &id[..pos];
    }
    id.trim_matches('/').to_string()
}

/// Validate that a Spotify ID is non-empty alphanumeric (typically 22 chars base62).
fn validate_spotify_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("Spotify ID is missing from the provided link.".to_string());
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(format!(
            "Invalid Spotify ID '{id}'. Spotify IDs must contain only alphanumeric characters."
        ));
    }
    Ok(())
}

/// Extract JSON embedded in `<script id="__NEXT_DATA__" type="application/json">` from HTML.
pub fn extract_embed_json(html: &str) -> Result<serde_json::Value, String> {
    // Look for <script id="__NEXT_DATA__"
    let script_start = html
        .find("<script id=\"__NEXT_DATA__\"")
        .or_else(|| html.find("<script id='__NEXT_DATA__'"))
        .or_else(|| html.find("id=\"__NEXT_DATA__\""));

    let script_pos = match script_start {
        Some(pos) => pos,
        None => {
            return Err("Could not locate __NEXT_DATA__ script in Spotify embed page.".to_string());
        }
    };

    // Find the end of opening script tag '>'
    let tag_close = match html[script_pos..].find('>') {
        Some(offset) => script_pos + offset + 1,
        None => {
            return Err("Malformed script tag in Spotify embed page.".to_string());
        }
    };

    // Find closing '</script>'
    let script_end = match html[tag_close..].find("</script>") {
        Some(offset) => tag_close + offset,
        None => {
            return Err("Unclosed __NEXT_DATA__ script in Spotify embed page.".to_string());
        }
    };

    let json_text = html[tag_close..script_end].trim();
    if json_text.is_empty() {
        return Err("Empty __NEXT_DATA__ payload in Spotify embed page.".to_string());
    }

    serde_json::from_str::<serde_json::Value>(json_text).map_err(|e| {
        format!("Failed to parse Spotify embed __NEXT_DATA__ JSON: {e}")
    })
}

/// Extract Open Graph metadata fallback (`og:title`, `og:description`, `og:image`) from HTML.
pub fn extract_open_graph_metadata(
    html: &str,
) -> (Option<String>, Option<String>, Option<String>) {
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut image: Option<String> = None;

    // Search for meta tags with property or name matching og:...
    for line in html.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("<meta") {
            continue;
        }

        let is_og_title = trimmed.contains("property=\"og:title\"")
            || trimmed.contains("property='og:title'")
            || trimmed.contains("name=\"og:title\"");
        let is_og_desc = trimmed.contains("property=\"og:description\"")
            || trimmed.contains("property='og:description'")
            || trimmed.contains("name=\"description\"");
        let is_og_image = trimmed.contains("property=\"og:image\"")
            || trimmed.contains("property='og:image'")
            || trimmed.contains("name=\"og:image\"");

        if is_og_title && title.is_none() {
            title = extract_meta_content(trimmed);
        } else if is_og_desc && description.is_none() {
            description = extract_meta_content(trimmed);
        } else if is_og_image && image.is_none() {
            image = extract_meta_content(trimmed);
        }

        if title.is_some() && description.is_some() && image.is_some() {
            break;
        }
    }

    // Clean up title if it contains Spotify branding (e.g. "Song Name - song by Artist | Spotify")
    let cleaned_title = title.map(|t| {
        let mut s = unescape_html_entities(&t);
        if let Some(pos) = s.rfind(" | Spotify") {
            s.truncate(pos);
        }
        s.trim().to_string()
    });

    let cleaned_desc = description.map(|d| unescape_html_entities(&d).trim().to_string());

    (cleaned_title, cleaned_desc, image)
}

fn extract_meta_content(meta_tag: &str) -> Option<String> {
    if let Some(pos) = meta_tag.find("content=\"") {
        let rest = &meta_tag[pos + 9..];
        if let Some(end) = rest.find('"') {
            return Some(rest[..end].to_string());
        }
    }
    if let Some(pos) = meta_tag.find("content='") {
        let rest = &meta_tag[pos + 9..];
        if let Some(end) = rest.find('\'') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// Parse `SpotifyImportResult` from the `__NEXT_DATA__` JSON structure.
pub fn parse_embed_entity(
    val: &serde_json::Value,
    target: &ParsedSpotifyTarget,
    original_url: &str,
) -> Result<SpotifyImportResult, String> {
    // Check for 404 or page not found inside JSON
    if let Some(status) = val.pointer("/props/pageProps/status").and_then(|v| v.as_i64()) {
        if status == 404 {
            return Err(
                "This Spotify link could not be found or is set to private. Please ensure the link is public."
                    .to_string(),
            );
        }
    }

    if let Some(page_title) = val.pointer("/props/pageProps/title").and_then(|v| v.as_str()) {
        if page_title.eq_ignore_ascii_case("Page not found") {
            return Err(
                "This Spotify link could not be found or is set to private. Please ensure the link is public."
                    .to_string(),
            );
        }
    }

    // Locate the entity object across common Next.js state shapes
    let entity = val
        .pointer("/props/pageProps/state/data/entity")
        .or_else(|| val.pointer("/props/pageProps/state/entity"))
        .or_else(|| val.pointer("/props/pageProps/entity"))
        .or_else(|| val.pointer("/props/pageProps/data/entity"))
        .ok_or_else(|| {
            "Unable to parse Spotify metadata: the page layout may have changed.".to_string()
        })?;

    // Extract title
    let title = entity["title"]
        .as_str()
        .or_else(|| entity["name"].as_str())
        .unwrap_or("Untitled Spotify Resource")
        .to_string();

    // Extract subtitle (often artist or creator)
    let subtitle = entity["subtitle"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| {
            entity["artists"].as_array().map(|artists| {
                artists
                    .iter()
                    .filter_map(|a| a["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        });

    // Extract cover art URL
    let cover_url = extract_image_url(entity);

    match target.resource_type {
        SpotifyResourceType::Track => {
            let artist = subtitle
                .clone()
                .unwrap_or_else(|| "Unknown Artist".to_string());
            let album = entity["album"]["name"]
                .as_str()
                .or_else(|| entity["albumTitle"].as_str())
                .unwrap_or(&title)
                .to_string();

            let duration_ms = entity["duration"]
                .as_f64()
                .or_else(|| entity["duration"].as_u64().map(|v| v as f64))
                .unwrap_or(0.0);
            let duration_secs = duration_ms / 1000.0;

            let track = StreamTrack::new(
                target.id.clone(),
                title.clone(),
                artist,
                album,
                duration_secs,
                cover_url.clone(),
                format!("spotify:track:{}", target.id),
                "spotify",
            );

            Ok(SpotifyImportResult {
                resource_type: SpotifyResourceType::Track,
                id: target.id.clone(),
                title,
                subtitle,
                cover_url,
                tracks: vec![track],
                total_tracks: 1,
                is_truncated: false,
                truncation_message: None,
                original_url: original_url.to_string(),
            })
        }
        SpotifyResourceType::Album | SpotifyResourceType::Playlist => {
            let mut tracks = Vec::new();
            if let Some(track_list) = entity["trackList"].as_array() {
                for (idx, item) in track_list.iter().enumerate() {
                    let item_title = item["title"]
                        .as_str()
                        .or_else(|| item["name"].as_str())
                        .unwrap_or("Unknown Track")
                        .to_string();

                    let item_artist = item["subtitle"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| {
                            item["artists"].as_array().map(|artists| {
                                artists
                                    .iter()
                                    .filter_map(|a| a["name"].as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            })
                        })
                        .or_else(|| subtitle.clone())
                        .unwrap_or_else(|| "Unknown Artist".to_string());

                    let d_ms = item["duration"]
                        .as_f64()
                        .or_else(|| item["duration"].as_u64().map(|v| v as f64))
                        .unwrap_or(0.0);
                    let duration_secs = d_ms / 1000.0;

                    let raw_uri = item["uri"].as_str().unwrap_or("");
                    let track_id = if raw_uri.starts_with("spotify:track:") {
                        raw_uri.trim_start_matches("spotify:track:").to_string()
                    } else if let Some(id_str) = item["id"].as_str() {
                        id_str.to_string()
                    } else if let Some(uid_str) = item["uid"].as_str() {
                        uid_str.to_string()
                    } else {
                        format!("{}_{}", target.id, idx + 1)
                    };

                    let item_uri = format!("spotify:track:{track_id}");
                    let item_album = if target.resource_type == SpotifyResourceType::Album {
                        title.clone()
                    } else {
                        item["album"]["name"]
                            .as_str()
                            .unwrap_or(&title)
                            .to_string()
                    };

                    let item_art = extract_image_url(item).or_else(|| cover_url.clone());

                    tracks.push(StreamTrack::new(
                        track_id,
                        item_title,
                        item_artist,
                        item_album,
                        duration_secs,
                        item_art,
                        item_uri,
                        "spotify",
                    ));
                }
            }

            // Truncation detection logic
            // Spotify embed pages cap at 100 tracks.
            let reported_total = entity["totalTracks"]
                .as_u64()
                .or_else(|| entity["totalCount"].as_u64())
                .or_else(|| entity["total"].as_u64())
                .map(|v| v as usize);

            let (total_tracks, is_truncated, truncation_message) = match reported_total {
                Some(total) if total > tracks.len() => (
                    total,
                    true,
                    Some(format!("Showing first {} of {} tracks", tracks.len(), total)),
                ),
                _ if tracks.len() >= 100 => (
                    tracks.len(),
                    true,
                    Some(format!("Showing first {} tracks", tracks.len())),
                ),
                _ => (tracks.len(), false, None),
            };

            Ok(SpotifyImportResult {
                resource_type: target.resource_type,
                id: target.id.clone(),
                title,
                subtitle,
                cover_url,
                tracks,
                total_tracks,
                is_truncated,
                truncation_message,
                original_url: original_url.to_string(),
            })
        }
    }
}

/// Helper to pick the best image URL from visualIdentity, coverArt, or images.
fn extract_image_url(val: &serde_json::Value) -> Option<String> {
    // 1. visualIdentity.image array
    if let Some(images) = val
        .pointer("/visualIdentity/image")
        .and_then(|v| v.as_array())
    {
        // Try to pick the image with highest resolution or the first
        let mut best_url: Option<String> = None;
        let mut max_dim: u64 = 0;
        for img in images {
            if let Some(url) = img["url"].as_str() {
                let w = img["maxWidth"].as_u64().unwrap_or(0);
                let h = img["maxHeight"].as_u64().unwrap_or(0);
                let dim = w.max(h);
                if dim >= max_dim || best_url.is_none() {
                    max_dim = dim;
                    best_url = Some(url.to_string());
                }
            }
        }
        if best_url.is_some() {
            return best_url;
        }
    }

    // 2. coverArt.sources array
    if let Some(sources) = val.pointer("/coverArt/sources").and_then(|v| v.as_array()) {
        if let Some(first) = sources.first().and_then(|s| s["url"].as_str()) {
            return Some(first.to_string());
        }
    }

    // 3. images array
    if let Some(images) = val["images"].as_array() {
        if let Some(first) = images.first().and_then(|s| s["url"].as_str()) {
            return Some(first.to_string());
        }
    }

    // 4. attributes
    if let Some(attrs) = val["attributes"].as_array() {
        for attr in attrs {
            if attr["key"].as_str() == Some("image_url")
                || attr["key"].as_str() == Some("header_image_url_desktop")
            {
                if let Some(url) = attr["value"].as_str() {
                    return Some(url.to_string());
                }
            }
        }
    }

    None
}

/// Parse metadata from Spotify oEmbed JSON response.
pub fn parse_oembed_json(
    val: &serde_json::Value,
    target: &ParsedSpotifyTarget,
    original_url: &str,
) -> Result<SpotifyImportResult, String> {
    let title = val["title"]
        .as_str()
        .unwrap_or("Untitled Spotify Resource")
        .to_string();

    let cover_url = val["thumbnail_url"].as_str().map(|s| s.to_string());

    let tracks = if target.resource_type == SpotifyResourceType::Track {
        let (track_title, track_artist) = if let Some((t, a)) = split_title_artist(&title) {
            (t, a)
        } else {
            (title.clone(), "Unknown Artist".to_string())
        };

        vec![StreamTrack::new(
            target.id.clone(),
            track_title,
            track_artist,
            title.clone(),
            0.0,
            cover_url.clone(),
            format!("spotify:track:{}", target.id),
            "spotify",
        )]
    } else {
        Vec::new()
    };

    let total_tracks = tracks.len();

    Ok(SpotifyImportResult {
        resource_type: target.resource_type,
        id: target.id.clone(),
        title,
        subtitle: None,
        cover_url,
        tracks,
        total_tracks,
        is_truncated: false,
        truncation_message: None,
        original_url: original_url.to_string(),
    })
}

/// Helper to split "Title - Artist" or "Title by Artist" strings if present.
fn split_title_artist(title: &str) -> Option<(String, String)> {
    if let Some((t, a)) = title.split_once(" - ") {
        return Some((t.trim().to_string(), a.trim().to_string()));
    }
    if let Some((t, a)) = title.split_once(" by ") {
        return Some((t.trim().to_string(), a.trim().to_string()));
    }
    None
}

/// Unescape standard HTML entities in titles and descriptions.
fn unescape_html_entities(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&#x27;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

/// Default public Spotify metadata fetcher with ureq and SQLite caching.
pub struct PublicSpotifyFetcher {
    db: Option<Database>,
    agent: ureq::Agent,
}

impl PublicSpotifyFetcher {
    /// Initialize with the default database and a configured ureq agent.
    pub fn new() -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build();

        Self {
            db: Database::open().ok(),
            agent,
        }
    }

    /// Initialize with a specific database instance (e.g. for testing).
    pub fn with_db(db: Database) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build();

        Self {
            db: Some(db),
            agent,
        }
    }

    /// Initialize with both custom database and custom ureq agent.
    pub fn with_db_and_agent(db: Option<Database>, agent: ureq::Agent) -> Self {
        Self { db, agent }
    }

    /// Internal network fetch for the embed HTML page.
    fn fetch_embed_html(&self, target: &ParsedSpotifyTarget) -> Result<String, String> {
        let embed_url = target.embed_url();
        let resp = self.agent.get(&embed_url).call();

        match resp {
            Ok(r) => r.into_string().map_err(|e| {
                format!("Failed to read Spotify response: {e}")
            }),
            Err(ureq::Error::Status(404, _)) => Err(
                "This Spotify link could not be found or is set to private. Please ensure the link is public."
                    .to_string(),
            ),
            Err(ureq::Error::Status(429, _)) => Err(
                "Spotify rate limit encountered (HTTP 429). Please try again shortly."
                    .to_string(),
            ),
            Err(ureq::Error::Status(code, _)) => Err(format!(
                "Spotify returned unexpected HTTP error status {code}."
            )),
            Err(ureq::Error::Transport(e)) => Err(format!(
                "Network error: unable to connect to Spotify ({e}). Please check your internet connection."
            )),
        }
    }

    /// Internal network fetch for the official Spotify oEmbed endpoint.
    fn fetch_oembed_metadata(
        &self,
        target: &ParsedSpotifyTarget,
        original_url: &str,
    ) -> Result<SpotifyImportResult, String> {
        let open_url = target.open_url();
        let oembed_url = format!(
            "https://open.spotify.com/oembed?url={}",
            urlencoding::encode(&open_url)
        );

        let resp = self.agent.get(&oembed_url).call();

        match resp {
            Ok(r) => {
                let json_val = r.into_json::<serde_json::Value>().map_err(|e| {
                    format!("Failed to parse Spotify oEmbed JSON: {e}")
                })?;
                parse_oembed_json(&json_val, target, original_url)
            }
            Err(ureq::Error::Status(404, _)) => Err(
                "This Spotify link could not be found or is set to private. Please ensure the link is public."
                    .to_string(),
            ),
            Err(ureq::Error::Status(429, _)) => Err(
                "Spotify rate limit encountered (HTTP 429). Please try again shortly."
                    .to_string(),
            ),
            Err(ureq::Error::Transport(e)) => Err(format!(
                "Network error: unable to connect to Spotify ({e}). Please check your internet connection."
            )),
            Err(ureq::Error::Status(code, _)) => Err(format!(
                "Spotify oEmbed returned HTTP status {code}."
            )),
        }
    }
}

impl Default for PublicSpotifyFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyPublicMetadataFetcher for PublicSpotifyFetcher {
    fn fetch_metadata(&self, input: &str) -> Result<SpotifyImportResult, String> {
        let target = parse_spotify_link(input)?;
        let canonical_key = target.canonical_uri();

        // 1. Check SQLite metadata cache
        if let Some(ref db) = self.db {
            if let Ok(Some(cached_json)) = db.get_stream_cache(&canonical_key) {
                if let Ok(cached_result) =
                    serde_json::from_str::<SpotifyImportResult>(&cached_json)
                {
                    return Ok(cached_result);
                }
            }
        }

        // 2. Fetch Embed HTML (Primary)
        let embed_res = self.fetch_embed_html(&target);

        let result = match embed_res {
            Ok(html) => {
                // Try parsing embedded __NEXT_DATA__ JSON
                match extract_embed_json(&html) {
                    Ok(json_val) => parse_embed_entity(&json_val, &target, input),
                    Err(_) => {
                        // Embedded JSON was not found or failed; attempt Open Graph + oEmbed fallback
                        let (og_title, og_desc, og_image) = extract_open_graph_metadata(&html);
                        if let Ok(mut oembed_res) = self.fetch_oembed_metadata(&target, input) {
                            if oembed_res.cover_url.is_none() {
                                oembed_res.cover_url = og_image;
                            }
                            if oembed_res.subtitle.is_none() {
                                oembed_res.subtitle = og_desc;
                            }
                            Ok(oembed_res)
                        } else if let Some(title) = og_title {
                            // Synthesize result from Open Graph tags
                            let tracks = if target.resource_type == SpotifyResourceType::Track {
                                let (t_title, t_artist) = split_title_artist(&title)
                                    .unwrap_or_else(|| (title.clone(), og_desc.clone().unwrap_or_else(|| "Unknown Artist".to_string())));

                                vec![StreamTrack::new(
                                    target.id.clone(),
                                    t_title,
                                    t_artist,
                                    title.clone(),
                                    0.0,
                                    og_image.clone(),
                                    format!("spotify:track:{}", target.id),
                                    "spotify",
                                )]
                            } else {
                                Vec::new()
                            };

                            let total_tracks = tracks.len();

                            Ok(SpotifyImportResult {
                                resource_type: target.resource_type,
                                id: target.id.clone(),
                                title,
                                subtitle: og_desc,
                                cover_url: og_image,
                                tracks,
                                total_tracks,
                                is_truncated: false,
                                truncation_message: None,
                                original_url: input.to_string(),
                            })
                        } else {
                            Err("Unable to parse Spotify metadata: the page layout may have changed.".to_string())
                        }
                    }
                }
            }
            Err(e) => {
                // If 404 or rate-limited, fail immediately with clear error
                if e.contains("could not be found or is set to private") || e.contains("429") {
                    return Err(e);
                }
                // Try oEmbed fallback before reporting failure
                self.fetch_oembed_metadata(&target, input).map_err(|_| e)
            }
        }?;

        // 3. Cache successfully fetched metadata in SQLite
        if let Some(ref db) = self.db {
            if let Ok(serialized) = serde_json::to_string(&result) {
                let _ = db.set_stream_cache(
                    &canonical_key,
                    "spotify",
                    target.resource_type.as_str(),
                    &serialized,
                );
            }
        }

        Ok(result)
    }
}

/// URL encoding helper for query strings
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::with_capacity(s.len());
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        encoded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spotify_track_urls_and_queries() {
        let urls = [
            "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT",
            "http://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT",
            "open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT",
            "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT?si=abc123xyz&context=spotify%3Aalbum%3A123",
            "https://open.spotify.com/embed/track/4cOdK2wGLETKBW3PvgPWqT",
            "https://open.spotify.com/intl-ja/track/4cOdK2wGLETKBW3PvgPWqT",
            "https://open.spotify.com/intl-de/embed/track/4cOdK2wGLETKBW3PvgPWqT?si=999",
            "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT/",
        ];

        for u in urls {
            let res = parse_spotify_link(u).unwrap_or_else(|e| panic!("Failed to parse '{u}': {e}"));
            assert_eq!(res.resource_type, SpotifyResourceType::Track, "URL: {u}");
            assert_eq!(res.id, "4cOdK2wGLETKBW3PvgPWqT", "URL: {u}");
            assert_eq!(res.canonical_uri(), "spotify:track:4cOdK2wGLETKBW3PvgPWqT");
            assert_eq!(
                res.embed_url(),
                "https://open.spotify.com/embed/track/4cOdK2wGLETKBW3PvgPWqT"
            );
        }
    }

    #[test]
    fn test_parse_spotify_album_urls() {
        let urls = [
            "https://open.spotify.com/album/4aawyAB9vmqN3uQ7FjRGTy",
            "open.spotify.com/album/4aawyAB9vmqN3uQ7FjRGTy?si=test",
            "https://open.spotify.com/embed/album/4aawyAB9vmqN3uQ7FjRGTy",
            "https://open.spotify.com/intl-fr/album/4aawyAB9vmqN3uQ7FjRGTy/",
        ];

        for u in urls {
            let res = parse_spotify_link(u).unwrap_or_else(|e| panic!("Failed to parse '{u}': {e}"));
            assert_eq!(res.resource_type, SpotifyResourceType::Album);
            assert_eq!(res.id, "4aawyAB9vmqN3uQ7FjRGTy");
            assert_eq!(res.canonical_uri(), "spotify:album:4aawyAB9vmqN3uQ7FjRGTy");
        }
    }

    #[test]
    fn test_parse_spotify_playlist_urls() {
        let urls = [
            "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M",
            "https://open.spotify.com/embed/playlist/37i9dQZF1DXcBWIGoYBM5M",
            "https://open.spotify.com/user/spotify/playlist/37i9dQZF1DXcBWIGoYBM5M",
            "https://open.spotify.com/intl-es/playlist/37i9dQZF1DXcBWIGoYBM5M?si=pl123",
        ];

        for u in urls {
            let res = parse_spotify_link(u).unwrap_or_else(|e| panic!("Failed to parse '{u}': {e}"));
            assert_eq!(res.resource_type, SpotifyResourceType::Playlist);
            assert_eq!(res.id, "37i9dQZF1DXcBWIGoYBM5M");
            assert_eq!(
                res.canonical_uri(),
                "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M"
            );
        }
    }

    #[test]
    fn test_parse_spotify_uris() {
        let track_uri = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";
        let target = parse_spotify_link(track_uri).expect("parse track uri");
        assert_eq!(target.resource_type, SpotifyResourceType::Track);
        assert_eq!(target.id, "4cOdK2wGLETKBW3PvgPWqT");

        let album_uri = "spotify:album:4aawyAB9vmqN3uQ7FjRGTy";
        let target_album = parse_spotify_link(album_uri).expect("parse album uri");
        assert_eq!(target_album.resource_type, SpotifyResourceType::Album);
        assert_eq!(target_album.id, "4aawyAB9vmqN3uQ7FjRGTy");

        let playlist_uri = "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M";
        let target_pl = parse_spotify_link(playlist_uri).expect("parse playlist uri");
        assert_eq!(target_pl.resource_type, SpotifyResourceType::Playlist);
        assert_eq!(target_pl.id, "37i9dQZF1DXcBWIGoYBM5M");

        let user_pl_uri = "spotify:user:spotify:playlist:37i9dQZF1DXcBWIGoYBM5M";
        let target_user_pl = parse_spotify_link(user_pl_uri).expect("parse user playlist uri");
        assert_eq!(target_user_pl.resource_type, SpotifyResourceType::Playlist);
        assert_eq!(target_user_pl.id, "37i9dQZF1DXcBWIGoYBM5M");
    }

    #[test]
    fn test_parse_invalid_links_and_errors() {
        assert!(parse_spotify_link("").is_err());
        assert!(parse_spotify_link("   ").is_err());
        assert!(parse_spotify_link("https://youtube.com/watch?v=123").is_err());
        assert!(parse_spotify_link("spotify:artist:0gxyHStUsqpMadRV0Di1Qt").is_err());
        assert!(parse_spotify_link("https://open.spotify.com/artist/0gxyHStUsqpMadRV0Di1Qt").is_err());
        assert!(parse_spotify_link("spotify:track:invalid_characters!@#$").is_err());
    }

    #[test]
    fn test_extract_embed_json_and_track_metadata() {
        let html_payload = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body>
            <script id="__NEXT_DATA__" type="application/json">
            {
                "props": {
                    "pageProps": {
                        "state": {
                            "data": {
                                "entity": {
                                    "type": "track",
                                    "title": "Never Gonna Give You Up",
                                    "subtitle": "Rick Astley",
                                    "duration": 213573,
                                    "id": "4cOdK2wGLETKBW3PvgPWqT",
                                    "uri": "spotify:track:4cOdK2wGLETKBW3PvgPWqT",
                                    "visualIdentity": {
                                        "image": [
                                            { "url": "https://example.com/art_small.jpg", "maxWidth": 64, "maxHeight": 64 },
                                            { "url": "https://example.com/art_large.jpg", "maxWidth": 640, "maxHeight": 640 }
                                        ]
                                    }
                                }
                            }
                        }
                    }
                }
            }
            </script>
        </body>
        </html>
        "#;

        let json_val = extract_embed_json(html_payload).expect("extract json");
        let target = ParsedSpotifyTarget {
            resource_type: SpotifyResourceType::Track,
            id: "4cOdK2wGLETKBW3PvgPWqT".to_string(),
        };

        let result = parse_embed_entity(&json_val, &target, "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT").expect("parse entity");
        assert_eq!(result.title, "Never Gonna Give You Up");
        assert_eq!(result.subtitle.as_deref(), Some("Rick Astley"));
        assert_eq!(result.cover_url.as_deref(), Some("https://example.com/art_large.jpg"));
        assert_eq!(result.tracks.len(), 1);
        assert_eq!(result.total_tracks, 1);
        assert!(!result.is_truncated);

        let t = &result.tracks[0];
        assert_eq!(t.title, "Never Gonna Give You Up");
        assert_eq!(t.artist, "Rick Astley");
        assert!((t.duration_secs - 213.573).abs() < 1e-3);
        assert_eq!(t.provider, "spotify");
        assert_eq!(t.uri, "spotify:track:4cOdK2wGLETKBW3PvgPWqT");
    }

    #[test]
    fn test_extract_album_with_multiple_tracks() {
        let html_payload = r#"
        <script id="__NEXT_DATA__" type="application/json">
        {
            "props": {
                "pageProps": {
                    "state": {
                        "data": {
                            "entity": {
                                "type": "album",
                                "title": "Discovery",
                                "subtitle": "Daft Punk",
                                "visualIdentity": {
                                    "image": [{ "url": "https://example.com/discovery.jpg", "maxWidth": 300, "maxHeight": 300 }]
                                },
                                "trackList": [
                                    { "title": "One More Time", "subtitle": "Daft Punk", "duration": 320000, "uri": "spotify:track:track1" },
                                    { "title": "Aerodynamic", "subtitle": "Daft Punk", "duration": 207000, "uri": "spotify:track:track2" },
                                    { "title": "Digital Love", "subtitle": "Daft Punk", "duration": 298000, "uri": "spotify:track:track3" }
                                ]
                            }
                        }
                    }
                }
            }
        }
        </script>
        "#;

        let json_val = extract_embed_json(html_payload).expect("extract album json");
        let target = ParsedSpotifyTarget {
            resource_type: SpotifyResourceType::Album,
            id: "album_123".to_string(),
        };

        let result = parse_embed_entity(&json_val, &target, "https://open.spotify.com/album/album_123").expect("parse album");
        assert_eq!(result.title, "Discovery");
        assert_eq!(result.tracks.len(), 3);
        assert_eq!(result.total_tracks, 3);
        assert!(!result.is_truncated);
        assert_eq!(result.tracks[0].title, "One More Time");
        assert_eq!(result.tracks[0].album, "Discovery");
        assert_eq!(result.tracks[0].art_url.as_deref(), Some("https://example.com/discovery.jpg"));
        assert_eq!(result.tracks[1].title, "Aerodynamic");
    }

    #[test]
    fn test_playlist_truncation_detection() {
        let mut track_items = Vec::new();
        for i in 0..100 {
            track_items.push(format!(
                r#"{{"title": "Track {i}", "subtitle": "Artist", "duration": 180000, "uri": "spotify:track:id_{i}"}}"#
            ));
        }
        let tracks_json = track_items.join(",");

        let html_payload = format!(
            r#"
            <script id="__NEXT_DATA__" type="application/json">
            {{
                "props": {{
                    "pageProps": {{
                        "state": {{
                            "data": {{
                                "entity": {{
                                    "type": "playlist",
                                    "title": "Mega Playlist",
                                    "totalTracks": 250,
                                    "trackList": [{tracks_json}]
                                }}
                            }}
                        }}
                    }}
                }}
            }}
            </script>
            "#
        );

        let json_val = extract_embed_json(&html_payload).expect("extract json");
        let target = ParsedSpotifyTarget {
            resource_type: SpotifyResourceType::Playlist,
            id: "mega_pl".to_string(),
        };

        let result = parse_embed_entity(&json_val, &target, "https://open.spotify.com/playlist/mega_pl").expect("parse playlist");
        assert_eq!(result.title, "Mega Playlist");
        assert_eq!(result.tracks.len(), 100);
        assert_eq!(result.total_tracks, 250);
        assert!(result.is_truncated);
        assert_eq!(
            result.truncation_message.as_deref(),
            Some("Showing first 100 of 250 tracks")
        );
    }

    #[test]
    fn test_embed_404_detection() {
        let html_404 = r#"
        <script id="__NEXT_DATA__" type="application/json">
        {
            "props": {
                "pageProps": {
                    "status": 404,
                    "title": "Page not found",
                    "description": "We can’t seem to find the page you are looking for."
                }
            }
        }
        </script>
        "#;

        let json_val = extract_embed_json(html_404).expect("extract 404 json");
        let target = ParsedSpotifyTarget {
            resource_type: SpotifyResourceType::Track,
            id: "0000000000000000000000".to_string(),
        };

        let err = parse_embed_entity(&json_val, &target, "https://open.spotify.com/track/0000000000000000000000").unwrap_err();
        assert!(err.contains("could not be found or is set to private"), "Actual err: {err}");
    }

    #[test]
    fn test_open_graph_fallback_extraction() {
        let html_og = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta property="og:title" content="Get Lucky - song by Daft Punk | Spotify" />
            <meta property="og:description" content="Listen to Get Lucky on Spotify. Daft Punk · Song · 2013." />
            <meta property="og:image" content="https://example.com/ram.jpg" />
        </head>
        <body></body>
        </html>
        "#;

        let (title, desc, image) = extract_open_graph_metadata(html_og);
        assert_eq!(title.as_deref(), Some("Get Lucky - song by Daft Punk"));
        assert_eq!(desc.as_deref(), Some("Listen to Get Lucky on Spotify. Daft Punk · Song · 2013."));
        assert_eq!(image.as_deref(), Some("https://example.com/ram.jpg"));
    }

    #[test]
    fn test_oembed_json_parsing() {
        let oembed_json = serde_json::json!({
            "title": "Starboy - The Weeknd",
            "thumbnail_url": "https://example.com/starboy.jpg",
            "type": "rich",
            "provider_name": "Spotify"
        });

        let target = ParsedSpotifyTarget {
            resource_type: SpotifyResourceType::Track,
            id: "starboy_id".to_string(),
        };

        let result = parse_oembed_json(&oembed_json, &target, "https://open.spotify.com/track/starboy_id").expect("parse oembed");
        assert_eq!(result.title, "Starboy - The Weeknd");
        assert_eq!(result.cover_url.as_deref(), Some("https://example.com/starboy.jpg"));
        assert_eq!(result.tracks.len(), 1);
        assert_eq!(result.tracks[0].title, "Starboy");
        assert_eq!(result.tracks[0].artist, "The Weeknd");
        assert_eq!(result.tracks[0].uri, "spotify:track:starboy_id");
    }

    #[test]
    fn test_sqlite_cache_integration() {
        let db = Database::open_in_memory().expect("open memory db");
        let fetcher = PublicSpotifyFetcher::with_db(db.clone());

        let target_uri = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";

        // Create a pre-cached result
        let cached_item = SpotifyImportResult {
            resource_type: SpotifyResourceType::Track,
            id: "4cOdK2wGLETKBW3PvgPWqT".to_string(),
            title: "Cached Never Gonna Give You Up".to_string(),
            subtitle: Some("Rick Astley".to_string()),
            cover_url: Some("https://example.com/cached.jpg".to_string()),
            tracks: vec![StreamTrack::new(
                "4cOdK2wGLETKBW3PvgPWqT",
                "Cached Never Gonna Give You Up",
                "Rick Astley",
                "Whenever You Need Somebody",
                213.0,
                Some("https://example.com/cached.jpg".to_string()),
                target_uri,
                "spotify",
            )],
            total_tracks: 1,
            is_truncated: false,
            truncation_message: None,
            original_url: "https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT".to_string(),
        };

        let serialized = serde_json::to_string(&cached_item).expect("serialize");
        db.set_stream_cache(target_uri, "spotify", "track", &serialized).expect("set cache");

        // Fetching through the fetcher should hit cache and NOT invoke network
        let fetched = fetcher.fetch_metadata("https://open.spotify.com/track/4cOdK2wGLETKBW3PvgPWqT").expect("fetch cached");
        assert_eq!(fetched.title, "Cached Never Gonna Give You Up");
        assert_eq!(fetched.tracks[0].title, "Cached Never Gonna Give You Up");
        assert_eq!(fetched.cover_url.as_deref(), Some("https://example.com/cached.jpg"));
    }
}
