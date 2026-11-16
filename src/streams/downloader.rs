use crate::library::model::Track;
use crate::library::Database;
use crate::streams::controller::StreamEvent;
use crate::streams::credentials::CredentialStore;
use crate::streams::playback::{
    build_match_query, resolve_youtube_match, YouTubeAlternativeMatch,
};
pub use crate::streams::provider::{DownloadFormat, DownloadJob, DownloadStatus, StreamTrack};
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::probe::Probe;
use lofty::tag::{Accessor, Tag, TagExt};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

/// Helper to construct a process Command with CREATE_NO_WINDOW on Windows and null stdin.
pub fn silent_command(program: impl AsRef<std::ffi::OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.stdin(std::process::Stdio::null());
    cmd
}

/// Sanitize artist and title strings so they are safe to use as Windows/POSIX file paths.
pub fn sanitize_filename(name: &str) -> String {
    let invalid_chars = ['\\', '/', ':', '*', '?', '"', '<', '>', '|', '\0', '\n', '\r'];
    let cleaned: String = name
        .chars()
        .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
        .collect();

    // Trim trailing and leading periods and spaces which are invalid on Windows
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        "track".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Retrieve the default download directory for audio files.
pub fn get_default_download_dir() -> PathBuf {
    // 1. Try first music folder configured in app config
    if let Some(folder) = crate::config::default_music_folders().first() {
        let p = PathBuf::from(folder);
        if p.exists() {
            let downloads = p.join("Downloads");
            let _ = fs::create_dir_all(&downloads);
            return downloads;
        }
    }

    // 2. Try user's Music or Downloads directory
    if let Some(home) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    {
        let music = home.join("Music");
        if music.exists() {
            let music_downloads = music.join("Downloads");
            let _ = fs::create_dir_all(&music_downloads);
            return music_downloads;
        }
        let downloads = home.join("Downloads");
        if downloads.exists() {
            return downloads;
        }
    }

    // 3. Fallback to app data downloads directory
    let fallback = crate::config::app_data_dir().join("downloads");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

/// Locate yt-dlp executable on the system PATH or common installation directories.
pub fn find_yt_dlp() -> Option<PathBuf> {
    // Check PATH via where.exe (Windows) or which (Unix)
    #[cfg(windows)]
    {
        if let Ok(output) = silent_command("where.exe").arg("yt-dlp").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let p = PathBuf::from(first_line.trim());
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }

        // Check common user Python Scripts directory
        if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
            let py_dir = home.join("AppData\\Local\\Programs\\Python");
            if py_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(py_dir) {
                    for entry in entries.flatten() {
                        let candidate = entry.path().join("Scripts\\yt-dlp.exe");
                        if candidate.is_file() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(output) = silent_command("which").arg("yt-dlp").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let p = PathBuf::from(first_line.trim());
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

/// Key-less YouTube video search fallback using yt-dlp or direct web scraping.
pub fn resolve_youtube_match_keyless(
    artist: &str,
    title: &str,
) -> Result<YouTubeAlternativeMatch, String> {
    let query = build_match_query(artist, title);

    // 1. Try yt-dlp JSON search if installed
    if let Some(yt_dlp) = find_yt_dlp() {
        let search_arg = format!("ytsearch1:{query}");
        let output = silent_command(yt_dlp)
            .args(["--dump-json", "--no-warnings", "--no-playlist", &search_arg])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(video_id) = val.get("id").and_then(|v| v.as_str()) {
                        let matched_title = val
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or(title)
                            .to_string();
                        let channel = val
                            .get("uploader")
                            .or_else(|| val.get("channel"))
                            .and_then(|v| v.as_str())
                            .unwrap_or(artist)
                            .to_string();
                        let duration = val
                            .get("duration")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);

                        return Ok(YouTubeAlternativeMatch {
                            video_id: video_id.to_string(),
                            title: matched_title,
                            channel_title: channel,
                            duration_secs: duration,
                            thumbnail_url: None,
                        });
                    }
                }
            }
        }
    }

    // 2. Fall back to scraping YouTube search results via HTTP
    scrape_youtube_search(&query, artist, title)
}

/// Fallback YouTube HTML scraper to resolve video ID without any API keys or CLI dependencies.
pub fn scrape_youtube_search(
    query: &str,
    fallback_artist: &str,
    fallback_title: &str,
) -> Result<YouTubeAlternativeMatch, String> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!("https://www.youtube.com/results?search_query={encoded}");

    let response = ureq::get(&url)
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .set("Accept-Language", "en-US,en;q=0.9")
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| format!("YouTube search scrape request failed: {e}"))?;

    let html = response
        .into_string()
        .map_err(|e| format!("Failed to read YouTube search response: {e}"))?;

    // Extract video ID from watch?v= links
    let mut search_pos = 0;
    while let Some(idx) = html[search_pos..].find("/watch?v=") {
        let start = search_pos + idx + 9;
        if start + 11 <= html.len() {
            let potential_id = &html[start..start + 11];
            if potential_id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Ok(YouTubeAlternativeMatch {
                    video_id: potential_id.to_string(),
                    title: format!("{fallback_artist} - {fallback_title}"),
                    channel_title: fallback_artist.to_string(),
                    duration_secs: 0.0,
                    thumbnail_url: None,
                });
            }
        }
        search_pos = start;
    }

    Err(format!("No YouTube video match found for query: '{query}'"))
}

/// Resolve a track pairing: first check SQLite cache, then YouTube Data API if key exists, then key-less fallback.
pub fn resolve_track_for_download(
    db: &Database,
    track: &StreamTrack,
) -> Result<YouTubeAlternativeMatch, String> {
    let cache_key = if !track.id.is_empty() {
        &track.id
    } else {
        &track.uri
    };

    // 1. Check SQLite cache
    if let Ok(Some(cached)) = db.get_stream_match(cache_key) {
        return Ok(YouTubeAlternativeMatch {
            video_id: cached.youtube_video_id,
            title: cached.title,
            channel_title: cached.channel,
            duration_secs: cached.duration_secs,
            thumbnail_url: track.art_url.clone(),
        });
    }

    // 2. Try YouTube Data API if key is configured
    if let Ok(Some(api_key)) = CredentialStore::get_youtube_api_key() {
        let trimmed = api_key.trim();
        if !trimmed.is_empty() {
            if let Ok(matched) = resolve_youtube_match(trimmed, &track.artist, &track.title) {
                let _ = db.set_stream_match(
                    cache_key,
                    &matched.video_id,
                    &matched.title,
                    &matched.channel_title,
                    matched.duration_secs,
                );
                return Ok(matched);
            }
        }
    }

    // 3. Fall back to key-less search/scrape
    let matched = resolve_youtube_match_keyless(&track.artist, &track.title)?;
    let _ = db.set_stream_match(
        cache_key,
        &matched.video_id,
        &matched.title,
        &matched.channel_title,
        matched.duration_secs,
    );
    Ok(matched)
}

/// Tag an audio file with Title, Artist, Album, and optional Front Cover Art using lofty.
pub fn tag_audio_file(
    path: &Path,
    title: &str,
    artist: &str,
    album: &str,
    art_bytes: Option<&[u8]>,
) -> Result<(), String> {
    tag_audio_file_with_genre(path, title, artist, album, art_bytes, None)
}

/// Tag an audio file with Title, Artist, Album, Genre, and optional Front Cover Art using lofty.
pub fn tag_audio_file_with_genre(
    path: &Path,
    title: &str,
    artist: &str,
    album: &str,
    art_bytes: Option<&[u8]>,
    genre: Option<&str>,
) -> Result<(), String> {
    let mut tagged_file = Probe::open(path)
        .map_err(|e| format!("Failed to open file for tagging: {e}"))?
        .read()
        .map_err(|e| format!("Failed to read tags: {e}"))?;

    let tag = match tagged_file.primary_tag_mut() {
        Some(t) => t,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(Tag::new(tag_type));
            tagged_file
                .primary_tag_mut()
                .ok_or_else(|| "Failed to initialize tag".to_string())?
        }
    };

    if !title.trim().is_empty() {
        tag.set_title(title.to_string());
    }
    if !artist.trim().is_empty() {
        tag.set_artist(artist.to_string());
    }
    if !album.trim().is_empty() {
        tag.set_album(album.to_string());
    }
    if let Some(g) = genre {
        let trimmed = g.trim();
        if !trimmed.is_empty() {
            tag.set_genre(trimmed.to_string());
        }
    }

    if let Some(art) = art_bytes {
        if !art.is_empty() {
            let pic = Picture::new_unchecked(
                PictureType::CoverFront,
                Some(MimeType::Jpeg),
                None,
                art.to_vec(),
            );
            tag.push_picture(pic);
        }
    }

    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Failed to write tags to audio file: {e}"))?;

    Ok(())
}

/// Tag a downloaded audio file from a StreamTrack and optional cover art bytes using lofty.
/// If cover art is not provided or empty, falls back to fetching missing cover art online.
/// Resolves missing genre online via genre_fetcher before persisting tags.
pub fn tag_downloaded_file(
    path: &Path,
    track: &StreamTrack,
    cover_art: Option<&[u8]>,
) -> Result<(), String> {
    let fallback_art;
    let art_bytes = match cover_art {
        Some(bytes) if !bytes.is_empty() => Some(bytes),
        _ => {
            fallback_art = crate::library::art_fetcher::fetch_missing_cover_art(
                &track.artist,
                &track.title,
                &track.album,
            );
            fallback_art.as_deref()
        }
    };

    let genre = crate::library::genre_fetcher::fetch_track_genre(&track.artist, &track.title);

    tag_audio_file_with_genre(
        path,
        &track.title,
        &track.artist,
        &track.album,
        art_bytes,
        genre.as_deref(),
    )
}

/// Maximum allowed size for streaming cache in bytes (500 MB).
pub const MAX_STREAM_CACHE_BYTES: u64 = 500 * 1024 * 1024;

/// Helper to extract clean track ID for caching and lookup.
pub fn extract_stream_track_id(track: &StreamTrack) -> String {
    if !track.id.is_empty() {
        track
            .id
            .strip_prefix("spotify:track:")
            .or_else(|| track.id.strip_prefix("stream:spotify:"))
            .or_else(|| track.id.strip_prefix(&format!("stream:{}:", track.provider)))
            .unwrap_or(&track.id)
            .to_string()
    } else if !track.uri.is_empty() {
        track
            .uri
            .strip_prefix("spotify:track:")
            .or_else(|| track.uri.strip_prefix("stream:spotify:"))
            .or_else(|| track.uri.strip_prefix(&format!("stream:{}:", track.provider)))
            .unwrap_or(&track.uri)
            .to_string()
    } else {
        sanitize_filename(&format!("{}_{}", track.artist, track.title))
    }
}

/// Retrieve the dedicated stream cache directory in the temporary directory.
pub fn get_stream_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("eclipse_stream_cache");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Check if an audio stream file is already cached and valid (> 1024 bytes).
/// If valid, touches the file to update its accessed/modified timestamp for LRU,
/// and returns Some((spotify_id, cache_path)). Otherwise returns None.
pub fn get_cached_stream_path(track: &StreamTrack) -> Option<(String, PathBuf)> {
    let spotify_id = extract_stream_track_id(track);
    let safe_id = sanitize_filename(&spotify_id);
    let cache_dir = get_stream_cache_dir();
    let cache_path = cache_dir.join(format!("eclipse_stream_{safe_id}.mp3"));

    if cache_path.is_file() {
        if let Ok(meta) = fs::metadata(&cache_path) {
            if meta.len() > 1024 {
                // Update accessed and modified timestamp so LRU knows it was recently used
                if let Ok(file) = fs::OpenOptions::new().write(true).open(&cache_path) {
                    let _ = file.set_modified(SystemTime::now());
                }
                return Some((spotify_id, cache_path));
            }
        }
    }
    None
}

/// Prune a specific cache directory using LRU (least recently used) eviction.
/// Deletes oldest files until total size is <= 90% of max_bytes (or 0 if max_bytes is 0).
pub fn prune_stream_cache_dir(dir: &Path, max_bytes: u64) -> Result<usize, String> {
    if !dir.is_dir() {
        return Ok(0);
    }

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read stream cache directory: {e}"))?;

    let mut files: Vec<(PathBuf, SystemTime, u64)> = Vec::new();
    let mut total_size: u64 = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                total_size += size;
                let ts = meta
                    .modified()
                    .or_else(|_| meta.accessed())
                    .unwrap_or(SystemTime::UNIX_EPOCH);
                files.push((path, ts, size));
            }
        }
    }

    if total_size <= max_bytes {
        return Ok(0);
    }

    // Target 90% of max_bytes to avoid pruning thrash
    let target_size = (max_bytes * 9) / 10;
    // Sort oldest timestamp first (LRU)
    files.sort_by_key(|(_, ts, _)| *ts);

    let mut deleted_count = 0;
    for (path, _, size) in files {
        if total_size <= target_size {
            break;
        }
        if fs::remove_file(&path).is_ok() {
            total_size = total_size.saturating_sub(size);
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}

/// Prune the stream cache if total size exceeds max_bytes.
/// Deletes oldest files one by one until total cache size is under 90% of max_bytes.
/// Returns the number of files deleted.
pub fn prune_stream_cache(max_bytes: u64) -> Result<usize, String> {
    prune_stream_cache_dir(&get_stream_cache_dir(), max_bytes)
}

/// Calculate total bytes currently consumed by files in a directory.
pub fn get_stream_cache_size_in_dir(dir: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut total: u64 = 0;
    for entry in entries.flatten() {
        if let Ok(meta) = entry.metadata() {
            if meta.is_file() {
                total += meta.len();
            }
        }
    }
    total
}

/// Calculate total bytes currently consumed by files in the stream cache directory.
pub fn get_stream_cache_size() -> u64 {
    get_stream_cache_size_in_dir(&get_stream_cache_dir())
}

/// Clear all cached stream audio files in a specific directory.
/// Returns total bytes freed.
pub fn clear_stream_cache_dir(dir: &Path) -> Result<u64, String> {
    if !dir.is_dir() {
        return Ok(0);
    }

    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read stream cache directory: {e}"))?;

    let mut freed_bytes: u64 = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(meta) = entry.metadata() {
                let size = meta.len();
                if fs::remove_file(&path).is_ok() {
                    freed_bytes += size;
                }
            }
        }
    }

    Ok(freed_bytes)
}

/// Clear all cached stream audio files in the stream cache directory.
/// Returns total bytes freed.
pub fn clear_stream_cache() -> Result<u64, String> {
    clear_stream_cache_dir(&get_stream_cache_dir())
}

/// Resolve and cache audio stream for a streaming track into a symphonia-compatible .mp3 file.
/// Returns (spotify_id, cache_path) on success.
pub fn cache_stream_track(track: &StreamTrack) -> Result<(String, PathBuf), String> {
    // 1. Instant cache lookup
    if let Some((spotify_id, cache_path)) = get_cached_stream_path(track) {
        return Ok((spotify_id, cache_path));
    }

    let spotify_id = extract_stream_track_id(track);
    let cache_dir = get_stream_cache_dir();
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create stream cache directory: {e}"))?;

    let safe_id = sanitize_filename(&spotify_id);
    let cache_path = cache_dir.join(format!("eclipse_stream_{safe_id}.mp3"));

    // 2. Resolve audio stream match (video_id)
    let video_id = if track.provider.eq_ignore_ascii_case("youtube") && !track.id.is_empty() && track.id.len() == 11 {
        track.id.clone()
    } else {
        let db = Database::open()
            .map_err(|e| format!("Failed to open library database: {e}"))?;
        let api_key = CredentialStore::get_youtube_api_key().ok().flatten();
        let matched = crate::streams::playback::resolve_or_cached_match(&db, api_key.as_deref(), track)
            .map_err(|e| format!("Failed to match stream track: {e}"))?;
        matched.video_id
    };

    // 3. Download audio stream using yt-dlp to cache_path
    let cancel_flag = AtomicBool::new(false);
    let saved_path = download_audio_with_ytdlp(&video_id, &cache_path, DownloadFormat::Mp3, &cancel_flag)?;

    // 4. Prune cache to maintain size limit (MAX_STREAM_CACHE_BYTES)
    let _ = prune_stream_cache(MAX_STREAM_CACHE_BYTES);

    Ok((spotify_id, saved_path))
}

/// Probe audio tags and properties for true sample rate, bit depth, and bitrate.
pub fn probe_track_audio_specs(path: &Path, format: DownloadFormat) -> (u32, u16, u32) {
    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
        let props = tagged_file.properties();
        let sample_rate = props.sample_rate().unwrap_or(0);
        let bitrate_kbps = props.audio_bitrate().or_else(|| props.overall_bitrate()).unwrap_or(0);
        let explicit_bit_depth = props.bit_depth().map(|b| b as u16);

        let is_lossless = matches!(format, DownloadFormat::Flac | DownloadFormat::Wav);
        let bit_depth = match explicit_bit_depth {
            Some(bd) if bd > 0 => bd,
            _ => {
                let is_hires = sample_rate >= 88200 || (is_lossless && (sample_rate >= 48000 || bitrate_kbps > 1500));
                if is_hires {
                    24
                } else {
                    16
                }
            }
        };

        let resolved_sr = if sample_rate > 0 {
            sample_rate
        } else if is_lossless {
            48000
        } else {
            44100
        };

        (resolved_sr, bit_depth, bitrate_kbps)
    } else {
        let is_lossless = matches!(format, DownloadFormat::Flac | DownloadFormat::Wav);
        if is_lossless {
            (48000, 24, 1536)
        } else {
            (44100, 16, 320)
        }
    }
}

/// Helper to extract all candidate streaming URIs and IDs for a stream track.
pub fn get_stream_candidate_uris(track: &StreamTrack) -> Vec<String> {
    let mut candidates = Vec::new();

    if !track.uri.trim().is_empty() {
        candidates.push(track.uri.trim().to_string());
    }
    if !track.id.trim().is_empty() {
        candidates.push(track.id.trim().to_string());
    }

    let clean_id = track
        .id
        .strip_prefix("spotify:track:")
        .or_else(|| track.id.strip_prefix("stream:spotify:"))
        .or_else(|| track.id.strip_prefix(&format!("stream:{}:", track.provider)))
        .unwrap_or(&track.id)
        .trim();

    let uri_clean_id = track
        .uri
        .strip_prefix("spotify:track:")
        .or_else(|| track.uri.strip_prefix("stream:spotify:"))
        .or_else(|| track.uri.strip_prefix(&format!("stream:{}:", track.provider)))
        .unwrap_or(&track.uri)
        .trim();

    for id_str in [clean_id, uri_clean_id] {
        if !id_str.is_empty() {
            candidates.push(format!("spotify:track:{id_str}"));
            candidates.push(format!("stream:spotify:{id_str}"));
            candidates.push(format!("stream:{}:{id_str}", track.provider));
            if !track.provider.eq_ignore_ascii_case("youtube") {
                candidates.push(format!("stream:youtube:{id_str}"));
            }
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

/// Auto-link saved playlists and tracks to newly downloaded local file path.
/// Upgrades matching stream track entries in SQLite to local tracks.
pub fn auto_link_downloaded_track_to_playlists(
    db: &Database,
    local_path: &Path,
    track: &StreamTrack,
    local_track_id: i64,
    format: DownloadFormat,
) -> usize {
    let candidates = get_stream_candidate_uris(track);
    let path_str = local_path.to_string_lossy().to_string();
    let format_str = format.label();
    let conn = db.lock_conn();
    let mut upgraded_count = 0;

    // Find all track rows in tracks table matching any candidate URI or URL
    let mut matching_track_ids = Vec::new();
    for cand in &candidates {
        if let Ok(mut stmt) = conn.prepare("SELECT id FROM tracks WHERE path = ?1 OR url = ?1") {
            if let Ok(rows) = stmt.query_map(rusqlite::params![cand], |r| r.get::<_, i64>(0)) {
                for id_res in rows.flatten() {
                    if id_res != local_track_id {
                        matching_track_ids.push(id_res);
                    }
                }
            }
        }
    }

    // Also match by partial clean id if long enough (e.g. spotify track ids are 22 chars)
    let clean_id = track
        .id
        .strip_prefix("spotify:track:")
        .or_else(|| track.id.strip_prefix("stream:spotify:"))
        .unwrap_or(&track.id)
        .trim();
    if clean_id.len() >= 8 {
        let pattern = format!("%{clean_id}%");
        if let Ok(mut stmt) =
            conn.prepare("SELECT id FROM tracks WHERE (path LIKE ?1 OR url LIKE ?1) AND id != ?2")
        {
            if let Ok(rows) = stmt.query_map(rusqlite::params![pattern, local_track_id], |r| {
                r.get::<_, i64>(0)
            }) {
                for id_res in rows.flatten() {
                    matching_track_ids.push(id_res);
                }
            }
        }
    }

    matching_track_ids.sort();
    matching_track_ids.dedup();

    // Re-link playlist_items to the new local track ID
    for old_id in &matching_track_ids {
        if let Ok(affected) = conn.execute(
            "UPDATE OR IGNORE playlist_items SET track_id = ?1 WHERE track_id = ?2",
            rusqlite::params![local_track_id, old_id],
        ) {
            upgraded_count += affected;
        }
        let _ = conn.execute(
            "DELETE FROM playlist_items WHERE track_id = ?1",
            rusqlite::params![old_id],
        );
        let _ = conn.execute(
            "DELETE FROM tracks WHERE id = ?1",
            rusqlite::params![old_id],
        );
    }

    // Ensure the local track row has format, path, and source = 'local'
    let _ = conn.execute(
        "UPDATE tracks SET path = ?1, format = ?2, source = 'local' WHERE id = ?3",
        rusqlite::params![path_str, format_str, local_track_id],
    );

    // If an optional playlist_tracks table exists in the schema, update it as well
    let has_playlist_tracks: bool = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='playlist_tracks'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);

    if has_playlist_tracks {
        for cand in &candidates {
            let _ = conn.execute(
                "UPDATE playlist_tracks SET path = ?1, format = ?2 WHERE path = ?3",
                rusqlite::params![path_str, format_str, cand],
            );
        }
    }

    upgraded_count
}

/// Index a newly downloaded and tagged audio file into the SQLite tracks database.
/// Returns (local_track_id, sample_rate, bit_depth).
pub fn index_downloaded_track(
    db: &Database,
    path: &Path,
    track: &StreamTrack,
    format: DownloadFormat,
) -> Result<(i64, u32, u16), String> {
    let (probed_sample_rate, probed_bit_depth, probed_bitrate) = probe_track_audio_specs(path, format);

    let mut inspected = crate::library::scanner::inspect_audio_file(path).unwrap_or_else(|| Track {
        path: path.to_string_lossy().to_string(),
        title: if !track.title.trim().is_empty() {
            track.title.clone()
        } else {
            "Unknown Title".to_string()
        },
        artist: track.artist.clone(),
        album_artist: track.artist.clone(),
        album: track.album.clone(),
        duration_secs: track.duration_secs,
        format: format.label().to_string(),
        folder: path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        sample_rate: probed_sample_rate,
        bitrate_kbps: probed_bitrate,
        ..Default::default()
    });

    inspected.format = format.label().to_string();
    if inspected.sample_rate == 0 && probed_sample_rate > 0 {
        inspected.sample_rate = probed_sample_rate;
    }
    if inspected.bitrate_kbps == 0 && probed_bitrate > 0 {
        inspected.bitrate_kbps = probed_bitrate;
    }

    // Ensure database record contains exact values so HI-RES / 24-BIT 48 KHZ is immediately available
    if probed_bit_depth >= 24 && matches!(format, DownloadFormat::Flac | DownloadFormat::Wav) {
        if inspected.bitrate_kbps <= 320 {
            inspected.bitrate_kbps = 1536;
        }
        if inspected.sample_rate == 0 {
            inspected.sample_rate = 48000;
        }
    }

    let local_path_str = path.to_string_lossy().to_string();
    let format_str = format.label();

    // If local_path does not exist yet in tracks, but a streaming row matches this track,
    // upgrade the existing streaming row in tracks directly so row ID and playlist links are preserved.
    let candidates = get_stream_candidate_uris(track);
    let existing_local_id = {
        let conn = db.lock_conn();
        conn.query_row(
            "SELECT id FROM tracks WHERE path = ?1",
            rusqlite::params![local_path_str],
            |r| r.get::<_, i64>(0),
        )
        .ok()
    };

    if existing_local_id.is_none() {
        let conn = db.lock_conn();
        for cand in &candidates {
            if let Ok(mut stmt) = conn.prepare("SELECT id FROM tracks WHERE path = ?1 OR url = ?1") {
                if let Ok(mut rows) = stmt.query(rusqlite::params![cand]) {
                    if let Ok(Some(row)) = rows.next() {
                        if let Ok(old_id) = row.get::<_, i64>(0) {
                            let _ = conn.execute(
                                "UPDATE tracks SET path = ?1, format = ?2, source = 'local' WHERE id = ?3",
                                rusqlite::params![local_path_str, format_str, old_id],
                            );
                            break;
                        }
                    }
                }
            }
        }
    }

    // Populate genre if missing or unknown prior to SQLite insertion
    if crate::library::genre_fetcher::is_missing_or_unknown_genre(&inspected.genre) {
        if let Some(genre) =
            crate::library::genre_fetcher::fetch_track_genre(&inspected.artist, &inspected.title)
        {
            let trimmed = genre.trim();
            if !trimmed.is_empty() {
                inspected.genre = trimmed.to_string();
                let _ = crate::library::genre_fetcher::embed_genre_in_file(path, trimmed);
            }
        }
    }

    let track_id = db
        .insert_track(&inspected)
        .map_err(|e| format!("Failed to insert track into SQLite library: {e}"))?;

    if !inspected.genre.is_empty() {
        let _ = db.update_track_genre(track_id, &inspected.genre);
    }

    // Auto-link any playlist references to this newly downloaded local file
    auto_link_downloaded_track_to_playlists(db, path, track, track_id, format);

    let final_sr = if inspected.sample_rate > 0 {
        inspected.sample_rate
    } else {
        probed_sample_rate
    };

    Ok((track_id, final_sr, probed_bit_depth))
}

/// Helper for indexing a downloaded track returning only the local track ID.
pub fn index_downloaded_track_id(
    db: &Database,
    path: &Path,
    track: &StreamTrack,
    format: DownloadFormat,
) -> Result<i64, String> {
    index_downloaded_track(db, path, track, format).map(|(id, _, _)| id)
}

/// Download audio stream via yt-dlp to destination path.
pub fn download_audio_with_ytdlp(
    video_id: &str,
    target_path: &Path,
    format: DownloadFormat,
    cancel_flag: &AtomicBool,
) -> Result<PathBuf, String> {
    let yt_dlp = find_yt_dlp().ok_or_else(|| {
        "yt-dlp is required to download and extract audio streams. Please install yt-dlp.".to_string()
    })?;

    if cancel_flag.load(Ordering::Relaxed) {
        return Err("Download cancelled by user".to_string());
    }

    let video_url = format!("https://www.youtube.com/watch?v={video_id}");
    let stem = target_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| sanitize_filename(video_id));
    let out_template = target_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{stem}.%(ext)s"));

    let ext = format.ext();

    let mut child = silent_command(yt_dlp)
        .args([
            "-x",
            "--audio-format",
            ext,
            "--audio-quality",
            "0",
            "-o",
            &out_template.to_string_lossy(),
            &video_url,
            "--no-playlist",
            "--no-warnings",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch yt-dlp: {e}"))?;

    // Poll child process while checking cancellation token
    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = child.kill();
            return Err("Download cancelled by user".to_string());
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(format!("yt-dlp exited with non-zero status: {:?}", status.code()));
                }
                break;
            }
            Ok(None) => {
                thread::sleep(std::time::Duration::from_millis(250));
            }
            Err(e) => {
                return Err(format!("Error waiting for yt-dlp process: {e}"));
            }
        }
    }

    // Determine the produced file (out_template with format.ext() extension)
    let produced_file = out_template.with_extension(ext);
    if produced_file.is_file() {
        // Rename to the requested target filename
        if produced_file != target_path {
            if target_path.exists() {
                let _ = fs::remove_file(target_path);
            }
            let _ = fs::rename(&produced_file, target_path);
        }
        return Ok(target_path.to_path_buf());
    }

    if target_path.is_file() {
        return Ok(target_path.to_path_buf());
    }

    Err(format!(
        "Audio download completed but output file was not found at {:?}",
        target_path
    ))
}

/// Strict concurrency cap for parallel background downloads.
pub const MAX_CONCURRENT_DOWNLOADS: usize = 2;

struct DownloadManagerInner {
    pending_queue: VecDeque<String>,
    running_jobs: HashSet<String>,
    jobs: HashMap<String, DownloadJob>,
    cancellations: HashMap<String, Arc<AtomicBool>>,
}

impl DownloadManagerInner {
    fn new() -> Self {
        Self {
            pending_queue: VecDeque::new(),
            running_jobs: HashSet::new(),
            jobs: HashMap::new(),
            cancellations: HashMap::new(),
        }
    }

    /// Extract up to (MAX_CONCURRENT_DOWNLOADS - running_jobs.len()) tasks from the pending queue.
    fn pop_next_tasks_to_start(
        &mut self,
    ) -> Vec<(String, StreamTrack, DownloadFormat, Arc<AtomicBool>)> {
        let mut tasks = Vec::new();
        while self.running_jobs.len() < MAX_CONCURRENT_DOWNLOADS {
            let Some(job_id) = self.pending_queue.pop_front() else {
                break;
            };

            let Some(job) = self.jobs.get_mut(&job_id) else {
                continue;
            };

            let cancel_flag = self
                .cancellations
                .entry(job_id.clone())
                .or_insert_with(|| Arc::new(AtomicBool::new(false)))
                .clone();

            if cancel_flag.load(Ordering::Relaxed) || job.is_failed() || job.is_completed() {
                continue;
            }

            self.running_jobs.insert(job_id.clone());
            tasks.push((job_id, job.track.clone(), job.format, cancel_flag));
        }
        tasks
    }
}

/// RAII Drop guard ensuring running_jobs slot is always released and next pending job is started.
struct WorkerGuard {
    inner: Arc<Mutex<DownloadManagerInner>>,
    event_tx: Sender<StreamEvent>,
    job_id: String,
    completed: bool,
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        let next_tasks = {
            let mut lock = self.inner.lock().unwrap();
            lock.running_jobs.remove(&self.job_id);
            lock.cancellations.remove(&self.job_id);
            if !self.completed {
                if let Some(job) = lock.jobs.get_mut(&self.job_id) {
                    if job.is_active() {
                        let err = "Download worker terminated unexpectedly".to_string();
                        job.status = DownloadStatus::Failed(err.clone());
                        job.error = Some(err.clone());
                        let _ = self.event_tx.send(StreamEvent::DownloadFailed {
                            job_id: self.job_id.clone(),
                            error: err,
                        });
                    }
                }
            }
            lock.pop_next_tasks_to_start()
        };

        for (jid, trk, fmt, flag) in next_tasks {
            spawn_download_worker(
                jid,
                trk,
                fmt,
                flag,
                self.inner.clone(),
                self.event_tx.clone(),
            );
        }
    }
}

fn spawn_download_worker(
    job_id: String,
    track: StreamTrack,
    format: DownloadFormat,
    cancel_flag: Arc<AtomicBool>,
    inner: Arc<Mutex<DownloadManagerInner>>,
    tx: Sender<StreamEvent>,
) {
    thread::Builder::new()
        .name(format!("downloader-{job_id}"))
        .spawn(move || {
            let mut guard = WorkerGuard {
                inner: inner.clone(),
                event_tx: tx.clone(),
                job_id: job_id.clone(),
                completed: false,
            };

            let update_job_status = |status: DownloadStatus| {
                let mut lock = inner.lock().unwrap();
                if let Some(job) = lock.jobs.get_mut(&job_id) {
                    job.status = status.clone();
                    let _ = tx.send(StreamEvent::DownloadUpdated(job.clone()));
                }
            };

            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                guard.completed = true;
                let _ = tx.send(StreamEvent::DownloadFailed {
                    job_id,
                    error: "Download cancelled".to_string(),
                });
                return;
            }

            // Stage 1: Resolving audio source
            update_job_status(DownloadStatus::Resolving);
            let db_opt = Database::open().ok();
            let matched = match &db_opt {
                Some(db) => resolve_track_for_download(db, &track),
                None => resolve_youtube_match_keyless(&track.artist, &track.title),
            };

            let matched_video = match matched {
                Ok(m) => m,
                Err(e) => {
                    guard.completed = true;
                    {
                        let mut lock = inner.lock().unwrap();
                        if let Some(job) = lock.jobs.get_mut(&job_id) {
                            job.status = DownloadStatus::Failed(e.clone());
                            job.error = Some(e.clone());
                        }
                    }
                    let _ = tx.send(StreamEvent::DownloadFailed {
                        job_id,
                        error: e,
                    });
                    return;
                }
            };

            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                guard.completed = true;
                let _ = tx.send(StreamEvent::DownloadFailed {
                    job_id,
                    error: "Download cancelled".to_string(),
                });
                return;
            }

            // Stage 2: Downloading audio file
            update_job_status(DownloadStatus::Downloading(0.0));
            let download_dir = get_default_download_dir();
            let filename = format!(
                "{} - {}.{}",
                sanitize_filename(&track.artist),
                sanitize_filename(&track.title),
                format.ext()
            );
            let target_path = download_dir.join(filename);

            let download_res = download_audio_with_ytdlp(
                &matched_video.video_id,
                &target_path,
                format,
                &cancel_flag,
            );

            let final_path = match download_res {
                Ok(p) => p,
                Err(e) => {
                    guard.completed = true;
                    {
                        let mut lock = inner.lock().unwrap();
                        if let Some(job) = lock.jobs.get_mut(&job_id) {
                            job.status = DownloadStatus::Failed(e.clone());
                            job.error = Some(e.clone());
                        }
                    }
                    let _ = tx.send(StreamEvent::DownloadFailed {
                        job_id,
                        error: e,
                    });
                    return;
                }
            };

            update_job_status(DownloadStatus::Downloading(1.0));

            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                guard.completed = true;
                let _ = fs::remove_file(&final_path);
                let _ = tx.send(StreamEvent::DownloadFailed {
                    job_id,
                    error: "Download cancelled".to_string(),
                });
                return;
            }

            // Stage 3: Tagging
            update_job_status(DownloadStatus::Tagging);

            let mut art_bytes = track.art_url.as_ref().and_then(|url| {
                ureq::get(url)
                    .timeout(std::time::Duration::from_secs(8))
                    .call()
                    .ok()
                    .and_then(|resp| {
                        let mut buf = Vec::new();
                        use std::io::Read;
                        resp.into_reader().read_to_end(&mut buf).ok().map(|_| buf)
                    })
            });

            // If track.art_url is None or failed to download, query online art fetcher fallback
            if art_bytes.is_none() {
                art_bytes = crate::library::art_fetcher::fetch_missing_cover_art(
                    &track.artist,
                    &track.title,
                    &track.album,
                );
            }

            if let Err(e) = tag_downloaded_file(&final_path, &track, art_bytes.as_deref()) {
                eprintln!("[WARN] Failed to tag audio file: {e}");
            }

            // Stage 4: Indexing into SQLite tracks table & Auto-linking to playlists
            let (local_track_id, sample_rate, bit_depth) = match Database::open() {
                Ok(db) => index_downloaded_track(&db, &final_path, &track, format)
                    .unwrap_or((0, 44100, 16)),
                Err(_) => (0, 44100, 16),
            };

            // Stage 5: Completed
            guard.completed = true;
            {
                let mut lock = inner.lock().unwrap();
                if let Some(job) = lock.jobs.get_mut(&job_id) {
                    job.status = DownloadStatus::Completed(final_path.clone());
                    job.track.set_local_match(local_track_id, final_path.to_string_lossy());
                }
            }

            let _ = tx.send(StreamEvent::DownloadCompleted {
                job_id,
                path: final_path,
                local_track_id,
                sample_rate,
                bit_depth,
            });
        })
        .ok();
}

/// Download manager managing async downloads with bounded concurrency.
pub struct DownloadManager {
    event_tx: Sender<StreamEvent>,
    inner: Arc<Mutex<DownloadManagerInner>>,
}

impl DownloadManager {
    pub fn new(event_tx: Sender<StreamEvent>) -> Self {
        Self {
            event_tx,
            inner: Arc::new(Mutex::new(DownloadManagerInner::new())),
        }
    }

    /// Submit a track to be resolved, downloaded, tagged, and indexed with bounded concurrency.
    pub fn start_download(&self, track: StreamTrack, format: DownloadFormat) {
        let job_id = if !track.id.is_empty() {
            track.id.clone()
        } else {
            track.uri.clone()
        };

        let cancel_flag = Arc::new(AtomicBool::new(false));
        let initial_job = DownloadJob::with_format(job_id.clone(), track, format);

        let next_tasks = {
            let mut lock = self.inner.lock().unwrap();
            lock.cancellations.insert(job_id.clone(), cancel_flag);
            lock.jobs.insert(job_id.clone(), initial_job.clone());
            if !lock.running_jobs.contains(&job_id) && !lock.pending_queue.contains(&job_id) {
                lock.pending_queue.push_back(job_id);
            }
            lock.pop_next_tasks_to_start()
        };

        let _ = self.event_tx.send(StreamEvent::DownloadUpdated(initial_job));

        for (jid, trk, fmt, flag) in next_tasks {
            spawn_download_worker(
                jid,
                trk,
                fmt,
                flag,
                self.inner.clone(),
                self.event_tx.clone(),
            );
        }
    }

    /// Submit a batch of tracks for background download with bounded concurrency.
    /// Returns the number of newly queued jobs.
    pub fn start_download_batch(&self, tracks: Vec<StreamTrack>, format: DownloadFormat) -> usize {
        let mut queued_count = 0;
        let mut events = Vec::new();

        let next_tasks = {
            let mut lock = self.inner.lock().unwrap();
            for track in tracks {
                let job_id = if !track.id.is_empty() {
                    track.id.clone()
                } else {
                    track.uri.clone()
                };

                // Skip if currently running or already in pending queue
                if lock.running_jobs.contains(&job_id) || lock.pending_queue.contains(&job_id) {
                    continue;
                }

                let cancel_flag = Arc::new(AtomicBool::new(false));
                let initial_job = DownloadJob::with_format(job_id.clone(), track, format);
                lock.cancellations.insert(job_id.clone(), cancel_flag);
                lock.jobs.insert(job_id.clone(), initial_job.clone());
                lock.pending_queue.push_back(job_id);
                events.push(initial_job);
                queued_count += 1;
            }
            lock.pop_next_tasks_to_start()
        };

        for ev in events {
            let _ = self.event_tx.send(StreamEvent::DownloadUpdated(ev));
        }

        for (jid, trk, fmt, flag) in next_tasks {
            spawn_download_worker(
                jid,
                trk,
                fmt,
                flag,
                self.inner.clone(),
                self.event_tx.clone(),
            );
        }

        queued_count
    }

    /// Submit a track for background download with default FLAC format.
    pub fn start_download_default(&self, track: StreamTrack) {
        self.start_download(track, DownloadFormat::default());
    }

    /// Cancel an in-progress or queued download.
    pub fn cancel_download(&self, id: &str) {
        let mut event_to_send = None;
        let next_tasks = {
            let mut lock = self.inner.lock().unwrap();
            if let Some(flag) = lock.cancellations.get(id) {
                flag.store(true, Ordering::Relaxed);
            }
            lock.pending_queue.retain(|jid| jid != id);
            if let Some(job) = lock.jobs.get_mut(id) {
                if job.is_active() {
                    job.status = DownloadStatus::Failed("Download cancelled".to_string());
                    job.error = Some("Download cancelled".to_string());
                    event_to_send = Some(job.clone());
                }
            }
            lock.pop_next_tasks_to_start()
        };

        if let Some(job) = event_to_send {
            let _ = self.event_tx.send(StreamEvent::DownloadUpdated(job));
        }
        let _ = self.event_tx.send(StreamEvent::DownloadFailed {
            job_id: id.to_string(),
            error: "Download cancelled".to_string(),
        });

        for (jid, trk, fmt, flag) in next_tasks {
            spawn_download_worker(
                jid,
                trk,
                fmt,
                flag,
                self.inner.clone(),
                self.event_tx.clone(),
            );
        }
    }

    /// Cancel all Queued, Resolving, and Downloading jobs.
    pub fn cancel_all_downloads(&self) {
        let mut cancelled_jobs = Vec::new();
        {
            let mut lock = self.inner.lock().unwrap();
            for flag in lock.cancellations.values() {
                flag.store(true, Ordering::Relaxed);
            }
            lock.pending_queue.clear();
            for job in lock.jobs.values_mut() {
                if job.is_active() {
                    job.status = DownloadStatus::Failed("Download cancelled".to_string());
                    job.error = Some("Download cancelled".to_string());
                    cancelled_jobs.push(job.clone());
                }
            }
        }

        for job in cancelled_jobs {
            let _ = self.event_tx.send(StreamEvent::DownloadUpdated(job.clone()));
            let _ = self.event_tx.send(StreamEvent::DownloadFailed {
                job_id: job.id,
                error: "Download cancelled".to_string(),
            });
        }
    }

    /// Retry a previously failed or completed download.
    pub fn retry_download(&self, id: &str) {
        let job_opt = {
            let lock = self.inner.lock().unwrap();
            lock.jobs.get(id).map(|j| (j.track.clone(), j.format))
        };

        if let Some((track, format)) = job_opt {
            self.start_download(track, format);
        }
    }

    /// Retry all failed downloads currently recorded.
    pub fn retry_all_failed(&self) {
        let failed_jobs: Vec<(StreamTrack, DownloadFormat)> = {
            let lock = self.inner.lock().unwrap();
            lock.jobs
                .values()
                .filter(|j| j.is_failed())
                .map(|j| (j.track.clone(), j.format))
                .collect()
        };

        for (track, format) in failed_jobs {
            self.start_download(track, format);
        }
    }

    /// Clear all completed jobs from memory.
    pub fn clear_completed(&self) {
        let mut lock = self.inner.lock().unwrap();
        lock.jobs.retain(|_, j| !matches!(j.status, DownloadStatus::Completed(_)));
    }

    /// Number of workers currently actively executing in threads (0..=MAX_CONCURRENT_DOWNLOADS).
    pub fn running_count(&self) -> usize {
        self.inner.lock().unwrap().running_jobs.len()
    }

    /// Number of jobs waiting in the pending queue.
    pub fn queued_count(&self) -> usize {
        self.inner.lock().unwrap().pending_queue.len()
    }

    /// Total count of active (running + queued) jobs.
    pub fn active_count(&self) -> usize {
        self.inner.lock().unwrap().jobs.values().filter(|j| j.is_active()).count()
    }

    /// Get a snapshot of a specific job by id.
    pub fn get_job(&self, id: &str) -> Option<DownloadJob> {
        self.inner.lock().unwrap().jobs.get(id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            sanitize_filename("AC/DC: Highway to Hell?"),
            "AC_DC_ Highway to Hell_"
        );
        assert_eq!(sanitize_filename("...Invalid..."), "Invalid");
        assert_eq!(sanitize_filename("Normal Title"), "Normal Title");
    }

    #[test]
    fn test_get_default_download_dir_not_empty() {
        let dir = get_default_download_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_download_job_status_transitions() {
        let track = StreamTrack::new(
            "test_1",
            "Song",
            "Artist",
            "Album",
            120.0,
            None,
            "spotify:track:test_1",
            "spotify",
        );
        let mut job = DownloadJob::new("test_1", track);
        assert_eq!(job.status, DownloadStatus::Queued);
        assert!(job.is_active());

        job.status = DownloadStatus::Resolving;
        assert!(job.is_active());

        job.status = DownloadStatus::Downloading(0.5);
        assert!(job.is_active());

        job.status = DownloadStatus::Tagging;
        assert!(job.is_active());

        job.status = DownloadStatus::Completed(PathBuf::from("C:\\music\\song.mp3"));
        assert!(!job.is_active());
        assert!(job.is_completed());

        job.status = DownloadStatus::Failed("Network error".to_string());
        assert!(!job.is_active());
        assert!(job.is_failed());
    }

    #[test]
    fn test_sqlite_track_indexing_with_insert_track() {
        let db = Database::open_in_memory().expect("open memory db");
        let track = StreamTrack::new(
            "dl_test_track",
            "Midnight City",
            "M83",
            "Hurry Up, We're Dreaming",
            243.0,
            Some("https://example.com/art.jpg".to_string()),
            "spotify:track:dl_test_track",
            "spotify",
        );

        let dummy_path = PathBuf::from("C:\\music\\Downloads\\M83 - Midnight City.mp3");
        let (id, sr, bd) = index_downloaded_track(&db, &dummy_path, &track, DownloadFormat::Mp3).expect("insert downloaded track");
        assert!(id > 0);
        assert_eq!(sr, 44100);
        assert_eq!(bd, 16);

        let retrieved = db.get_track_by_id(id).expect("query track").expect("track exists");
        assert_eq!(retrieved.title, "Midnight City");
        assert_eq!(retrieved.artist, "M83");
        assert_eq!(retrieved.album, "Hurry Up, We're Dreaming");
        assert_eq!(retrieved.duration_secs, 243.0);
        assert_eq!(retrieved.format, "MP3");

        // Verify indexing FLAC format sets 24-bit / 48 kHz / HI-RES
        let flac_path = PathBuf::from("C:\\music\\Downloads\\M83 - Midnight City.flac");
        let (flac_id, flac_sr, flac_bd) = index_downloaded_track(&db, &flac_path, &track, DownloadFormat::Flac).expect("insert flac track");
        assert!(flac_id > 0);
        assert_eq!(flac_sr, 48000);
        assert_eq!(flac_bd, 24);
        let flac_row = db.get_track_by_id(flac_id).unwrap().unwrap();
        assert_eq!(flac_row.format, "FLAC");
        assert!(flac_row.is_hires());
        assert_eq!(flac_row.quality_badge_text(), "HI-RES");

        // Verify indexing across all formats sets format = format.label()
        for fmt in DownloadFormat::ALL {
            let path = PathBuf::from(format!("C:\\music\\Downloads\\test.{}", fmt.ext()));
            let (track_id, _, _) = index_downloaded_track(&db, &path, &track, fmt).expect("insert format track");
            let row = db.get_track_by_id(track_id).unwrap().unwrap();
            assert_eq!(row.format, fmt.label());
        }
    }

    #[test]
    fn test_resolve_track_for_download_cached() {
        let db = Database::open_in_memory().expect("open memory db");
        let track = StreamTrack::new(
            "cached_dl_track",
            "Instant Crush",
            "Daft Punk",
            "Random Access Memories",
            337.0,
            None,
            "spotify:track:cached_dl_track",
            "spotify",
        );

        // Pre-populate match cache
        db.set_stream_match(
            &track.id,
            "a5uQMwRMHcs",
            "Daft Punk - Instant Crush ft. Julian Casablancas",
            "Daft Punk",
            337.0,
        )
        .expect("set stream match");

        let resolved = resolve_track_for_download(&db, &track).expect("resolve from cache");
        assert_eq!(resolved.video_id, "a5uQMwRMHcs");
        assert_eq!(resolved.channel_title, "Daft Punk");
    }

    #[test]
    fn test_download_manager_queue_and_cancellation() {
        let (tx, rx) = std::sync::mpsc::channel();
        let manager = DownloadManager::new(tx);

        let track = StreamTrack::new(
            "cancel_job_1",
            "Some Song",
            "Some Artist",
            "Some Album",
            180.0,
            None,
            "spotify:track:cancel_job_1",
            "spotify",
        );

        manager.cancel_download(&track.id);

        let mut got_cancelled = false;
        while let Ok(ev) = rx.try_recv() {
            if let StreamEvent::DownloadFailed { job_id, error } = ev {
                if job_id == "cancel_job_1" && error.contains("cancelled") {
                    got_cancelled = true;
                    break;
                }
            }
        }
        assert!(got_cancelled);
    }

    #[test]
    fn test_silent_command_construction() {
        let cmd = silent_command("echo");
        assert_eq!(cmd.get_program(), "echo");
    }

    #[test]
    fn test_stream_cache_dir_and_reuse() {
        let cache_dir = get_stream_cache_dir();
        assert!(cache_dir.is_dir());

        let spotify_id = "test_stream_cache_id_999";
        let cache_file = cache_dir.join(format!("eclipse_stream_{spotify_id}.mp3"));
        // Write dummy cached audio data (> 1024 bytes)
        let dummy_data = vec![0u8; 2048];
        fs::write(&cache_file, &dummy_data).expect("write dummy cache file");

        let track = StreamTrack::new(
            spotify_id,
            "Cache Test Track",
            "Cache Artist",
            "Cache Album",
            150.0,
            None,
            format!("spotify:track:{spotify_id}"),
            "spotify",
        );

        let (returned_id, returned_path) = cache_stream_track(&track).expect("cache_stream_track reuse");
        assert_eq!(returned_id, spotify_id);
        assert_eq!(returned_path, cache_file);
        assert!(returned_path.is_file());

        let _ = fs::remove_file(&cache_file);
    }

    #[test]
    fn test_probe_track_audio_specs_and_hires_inference() {
        let db = Database::open_in_memory().expect("open memory db");
        let dummy_flac = PathBuf::from("C:\\music\\Downloads\\hires_sample.flac");
        let (sr, bd, br) = probe_track_audio_specs(&dummy_flac, DownloadFormat::Flac);
        assert_eq!(sr, 48000);
        assert_eq!(bd, 24);
        assert!(br > 0);

        let dummy_mp3 = PathBuf::from("C:\\music\\Downloads\\std_sample.mp3");
        let (sr_mp3, bd_mp3, _) = probe_track_audio_specs(&dummy_mp3, DownloadFormat::Mp3);
        assert_eq!(sr_mp3, 44100);
        assert_eq!(bd_mp3, 16);

        let track = StreamTrack::new(
            "hires_dl_track",
            "Hi-Res Track",
            "Audiophile Artist",
            "Master Album",
            200.0,
            None,
            "spotify:track:hires_dl_track",
            "spotify",
        );

        let (track_id, track_sr, track_bd) =
            index_downloaded_track(&db, &dummy_flac, &track, DownloadFormat::Flac).expect("index flac");
        assert!(track_id > 0);
        assert_eq!(track_sr, 48000);
        assert_eq!(track_bd, 24);

        let row = db.get_track_by_id(track_id).unwrap().unwrap();
        assert_eq!(row.format, "FLAC");
        assert!(row.is_hires());
        assert_eq!(row.quality_badge_text(), "HI-RES");
    }

    #[test]
    fn test_auto_link_saved_playlists_upgrade_stream_to_local() {
        let db = Database::open_in_memory().expect("open memory db");
        let track = StreamTrack::new(
            "test_auto_link_spotify_1",
            "Get Lucky",
            "Daft Punk",
            "Random Access Memories",
            248.0,
            None,
            "spotify:track:test_auto_link_spotify_1",
            "spotify",
        );

        // 1. Simulate saving imported playlist with stream-only track
        let pl_id = db.create_playlist("Retro Waves").expect("create playlist");
        let stream_track_id = crate::streams::matcher::insert_or_get_stream_track(&db, &track)
            .expect("insert stream track");
        db.add_track_to_playlist(pl_id, stream_track_id)
            .expect("add track to playlist");

        // Verify initial state: playlist contains 1 track which is STREAM
        let initial_pl_tracks = db.get_playlist_tracks(pl_id).expect("query playlist tracks");
        assert_eq!(initial_pl_tracks.len(), 1);
        assert_eq!(initial_pl_tracks[0].path, "spotify:track:test_auto_link_spotify_1");
        assert!(initial_pl_tracks[0].is_stream());

        // 2. Perform indexing of newly downloaded track
        let downloaded_local_path = PathBuf::from("C:\\music\\Downloads\\Daft Punk - Get Lucky.flac");
        let (local_id, sr, bd) = index_downloaded_track(
            &db,
            &downloaded_local_path,
            &track,
            DownloadFormat::Flac,
        )
        .expect("index downloaded track");

        assert!(local_id > 0);
        assert_eq!(sr, 48000);
        assert_eq!(bd, 24);

        // 3. Verify that the playlist track has been automatically upgraded from STREAM to LOCAL
        let upgraded_pl_tracks = db.get_playlist_tracks(pl_id).expect("query playlist tracks after dl");
        assert_eq!(upgraded_pl_tracks.len(), 1);
        assert_eq!(
            upgraded_pl_tracks[0].path,
            downloaded_local_path.to_string_lossy()
        );
        assert_eq!(upgraded_pl_tracks[0].format, "FLAC");
        assert!(!upgraded_pl_tracks[0].is_stream(), "Playlist track must be converted from STREAM to LOCAL");
    }

    #[test]
    fn test_bounded_concurrency_scheduler_max_2_and_batch() {
        let (tx, rx) = std::sync::mpsc::channel();
        let manager = DownloadManager::new(tx);

        assert_eq!(MAX_CONCURRENT_DOWNLOADS, 2);

        let tracks: Vec<StreamTrack> = (1..=5)
            .map(|i| {
                StreamTrack::new(
                    format!("batch_track_{i}"),
                    format!("Batch Title {i}"),
                    "Batch Artist",
                    "Batch Album",
                    180.0,
                    None,
                    format!("spotify:track:batch_track_{i}"),
                    "spotify",
                )
            })
            .collect();

        // Start batch
        let queued = manager.start_download_batch(tracks, DownloadFormat::Flac);
        assert_eq!(queued, 5);

        // Verify that running workers count does not exceed MAX_CONCURRENT_DOWNLOADS (2)
        let running = manager.running_count();
        assert!(running <= MAX_CONCURRENT_DOWNLOADS);

        // Cancel all downloads
        manager.cancel_all_downloads();
        assert_eq!(manager.queued_count(), 0);

        // Drain channel events
        let mut cancel_event_count = 0;
        while let Ok(ev) = rx.try_recv() {
            if let StreamEvent::DownloadFailed { error, .. } = ev {
                if error.contains("cancelled") {
                    cancel_event_count += 1;
                }
            }
        }
        assert!(cancel_event_count > 0);
    }

    #[test]
    fn test_get_cached_stream_path_and_timestamp_update() {
        let cache_dir = get_stream_cache_dir();
        let spotify_id = "test_lookup_spec_123";
        let cache_file = cache_dir.join(format!("eclipse_stream_{spotify_id}.mp3"));
        let _ = fs::remove_file(&cache_file);

        let track = StreamTrack::new(
            spotify_id,
            "Lookup Title",
            "Lookup Artist",
            "Lookup Album",
            120.0,
            None,
            format!("spotify:track:{spotify_id}"),
            "spotify",
        );

        // 1. Missing cache file -> returns None
        assert!(get_cached_stream_path(&track).is_none());

        // 2. Cache file too small (<= 1024 bytes) -> returns None
        let small_data = vec![0u8; 512];
        fs::write(&cache_file, &small_data).expect("write small dummy file");
        assert!(get_cached_stream_path(&track).is_none());

        // 3. Cache file valid (> 1024 bytes) -> returns Some and updates timestamp
        let valid_data = vec![0u8; 2048];
        fs::write(&cache_file, &valid_data).expect("write valid dummy file");

        // Backdate modified time by 60 seconds
        let past_time = SystemTime::now() - Duration::from_secs(60);
        if let Ok(f) = fs::OpenOptions::new().write(true).open(&cache_file) {
            let _ = f.set_modified(past_time);
        }

        let res = get_cached_stream_path(&track);
        assert!(res.is_some());
        let (ret_id, ret_path) = res.unwrap();
        assert_eq!(ret_id, spotify_id);
        assert_eq!(ret_path, cache_file);

        // Verify that timestamp was touched/updated recently
        if let Ok(meta) = fs::metadata(&cache_file) {
            if let Ok(mtime) = meta.modified() {
                assert!(mtime > past_time);
            }
        }

        let _ = fs::remove_file(&cache_file);
    }

    #[test]
    fn test_prune_stream_cache_lru_eviction() {
        let temp_test_dir = std::env::temp_dir().join("eclipse_test_prune_lru");
        let _ = fs::create_dir_all(&temp_test_dir);

        let f1 = temp_test_dir.join("eclipse_stream_file1.mp3");
        let f2 = temp_test_dir.join("eclipse_stream_file2.mp3");
        let f3 = temp_test_dir.join("eclipse_stream_file3.mp3");

        // 1000 bytes each
        fs::write(&f1, vec![1u8; 1000]).unwrap();
        fs::write(&f2, vec![2u8; 1000]).unwrap();
        fs::write(&f3, vec![3u8; 1000]).unwrap();

        let now = SystemTime::now();
        // f1 oldest, f2 middle, f3 newest
        if let Ok(f) = fs::OpenOptions::new().write(true).open(&f1) {
            let _ = f.set_modified(now - Duration::from_secs(300));
        }
        if let Ok(f) = fs::OpenOptions::new().write(true).open(&f2) {
            let _ = f.set_modified(now - Duration::from_secs(200));
        }
        if let Ok(f) = fs::OpenOptions::new().write(true).open(&f3) {
            let _ = f.set_modified(now - Duration::from_secs(100));
        }

        assert_eq!(get_stream_cache_size_in_dir(&temp_test_dir), 3000);

        // When max_bytes is 3500 (total 3000 <= 3500), prune does nothing
        let deleted = prune_stream_cache_dir(&temp_test_dir, 3500).unwrap();
        assert_eq!(deleted, 0);
        assert!(f1.exists() && f2.exists() && f3.exists());

        // When max_bytes is 2500 (total 3000 > 2500), target is 2500 * 9 / 10 = 2250 bytes.
        // It must delete f1 (oldest, 1000 bytes), remaining is 2000 bytes <= 2250.
        let deleted = prune_stream_cache_dir(&temp_test_dir, 2500).unwrap();
        assert_eq!(deleted, 1);
        assert!(!f1.exists(), "Oldest file f1 must be pruned");
        assert!(f2.exists());
        assert!(f3.exists());
        assert_eq!(get_stream_cache_size_in_dir(&temp_test_dir), 2000);

        let _ = fs::remove_dir_all(&temp_test_dir);
    }

    #[test]
    fn test_clear_stream_cache_and_size_calculation() {
        let temp_test_dir = std::env::temp_dir().join("eclipse_test_clear_cache");
        let _ = fs::create_dir_all(&temp_test_dir);

        let f1 = temp_test_dir.join("eclipse_stream_alpha.mp3");
        let f2 = temp_test_dir.join("eclipse_stream_beta.mp3");

        fs::write(&f1, vec![0u8; 1500]).unwrap();
        fs::write(&f2, vec![0u8; 2500]).unwrap();

        assert_eq!(get_stream_cache_size_in_dir(&temp_test_dir), 4000);

        let freed = clear_stream_cache_dir(&temp_test_dir).unwrap();
        assert_eq!(freed, 4000);
        assert_eq!(get_stream_cache_size_in_dir(&temp_test_dir), 0);
        assert!(!f1.exists());
        assert!(!f2.exists());

        let _ = fs::remove_dir_all(&temp_test_dir);
    }

    #[test]
    fn test_public_stream_cache_functions() {
        let _ = prune_stream_cache(MAX_STREAM_CACHE_BYTES);
        let cache_dir = get_stream_cache_dir();
        let test_file = cache_dir.join("eclipse_stream_pub_test.mp3");
        fs::write(&test_file, vec![0u8; 1200]).unwrap();
        assert!(get_stream_cache_size() >= 1200);

        let track = StreamTrack::new(
            "pub_test",
            "Pub Track",
            "Pub Artist",
            "Pub Album",
            100.0,
            None,
            "spotify:track:pub_test",
            "spotify",
        );
        let cached = get_cached_stream_path(&track);
        assert!(cached.is_some());

        let freed = clear_stream_cache();
        assert!(freed.is_ok());
        assert!(!test_file.exists());
    }
}
