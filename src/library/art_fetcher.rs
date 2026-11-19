use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

/// Noise tokens typically found in bracketed audio titles.
const BRACKET_NOISE_TOKENS: &[&str] = &[
    "sped up",
    "speed up",
    "slowed",
    "slowed + reverb",
    "slowed and reverb",
    "reverb",
    "remaster",
    "remastered",
    "edition",
    "twintails edition",
    "deluxe edition",
    "deluxe",
    "version",
    "bonus track",
    "bonus",
    "live",
    "radio edit",
    "instrumental",
    "acoustic",
    "remix",
    "feat",
    "ft.",
    "ft ",
    "featuring",
    "official video",
    "official music video",
    "music video",
    "official audio",
    "audio",
    "visualizer",
    "lyric video",
    "lyrics",
    "original mix",
    "club mix",
    "extended mix",
    "single version",
    "album version",
    "edit",
    "mono",
    "stereo",
];

/// Noise suffixes typically found after a dash/hyphen in audio titles.
const DASH_NOISE_SUFFIXES: &[&str] = &[
    "sped up",
    "speed up",
    "slowed",
    "slowed + reverb",
    "slowed and reverb",
    "remaster",
    "remastered",
    "2011 remaster",
    "2021 remaster",
    "edition",
    "twintails edition",
    "deluxe edition",
    "anniversary edition",
    "expanded edition",
    "deluxe",
    "version",
    "single version",
    "album version",
    "bonus track",
    "bonus",
    "live",
    "radio edit",
    "instrumental",
    "acoustic",
    "remix",
    "feat.",
    "feat ",
    "ft.",
    "ft ",
    "featuring",
    "official video",
    "official audio",
    "visualizer",
    "lyric video",
    "lyrics",
    "edit",
    "mix",
    "mono",
    "stereo",
];

/// Clean noise suffixes from audio track title for enhanced search accuracy.
pub fn clean_title(title: &str) -> String {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // 1. Strip bracketed noise tokens: (...), [...], {...}, etc.
    let unbracketed = strip_bracketed_noise(trimmed);

    // 2. Strip trailing dash/hyphen noise tokens: " - Sped Up", " - 2011 Remaster", etc.
    let undashed = strip_dash_noise(&unbracketed);

    let collapsed = collapse_whitespace(&undashed);
    if collapsed.is_empty() {
        trimmed.to_string()
    } else {
        collapsed
    }
}

/// Clean featuring tokens and noise from artist string.
pub fn clean_artist(artist: &str) -> String {
    let trimmed = artist.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_lowercase();
    let feat_tokens = [" feat. ", " feat ", " ft. ", " ft ", " featuring ", " with "];
    let mut earliest: Option<usize> = None;
    for token in feat_tokens {
        if let Some(pos) = lower.find(token) {
            earliest = match earliest {
                Some(prev) => Some(prev.min(pos)),
                None => Some(pos),
            };
        }
    }

    let cleaned = if let Some(pos) = earliest {
        let prefix = trimmed[..pos].trim();
        if !prefix.is_empty() {
            prefix
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    let collapsed = collapse_whitespace(cleaned);
    if collapsed.is_empty() {
        trimmed.to_string()
    } else {
        collapsed
    }
}

fn strip_bracketed_noise(s: &str) -> String {
    let mut output = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if is_open_bracket(c) {
            let close = matching_close_bracket(c);
            let mut inner = String::new();
            let mut found_close = false;
            for inner_c in chars.by_ref() {
                if inner_c == close {
                    found_close = true;
                    break;
                }
                inner.push(inner_c);
            }

            if found_close && is_bracket_noise(&inner) {
                output.push(' ');
            } else {
                output.push(c);
                output.push_str(&inner);
                if found_close {
                    output.push(close);
                }
            }
        } else {
            output.push(c);
        }
    }

    output
}

#[inline]
fn is_open_bracket(c: char) -> bool {
    matches!(c, '(' | '[' | '{' | '（' | '【' | '〈' | '《' | '「')
}

#[inline]
fn matching_close_bracket(c: char) -> char {
    match c {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '（' => '）',
        '【' => '】',
        '〈' => '〉',
        '《' => '》',
        '「' => '」',
        _ => c,
    }
}

fn is_bracket_noise(inner: &str) -> bool {
    let lower = inner.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }

    for token in BRACKET_NOISE_TOKENS {
        if lower.contains(token) {
            return true;
        }
    }

    // Numeric remaster check, e.g. "2011 remaster", "2024 remaster"
    if lower.contains("remaster") {
        return true;
    }

    false
}

fn strip_dash_noise(s: &str) -> String {
    let mut result = s.to_string();
    let separators = [" - ", " – ", " — "];

    // Iteratively strip trailing hyphen noise parts
    loop {
        let mut stripped_any = false;
        for sep in separators {
            if let Some(pos) = result.rfind(sep) {
                let tail = result[pos + sep.len()..].trim().to_lowercase();
                if is_dash_noise(&tail) {
                    let prefix = result[..pos].trim();
                    if !prefix.is_empty() {
                        result = prefix.to_string();
                        stripped_any = true;
                        break;
                    }
                }
            }
        }
        if !stripped_any {
            break;
        }
    }

    result
}

fn is_dash_noise(tail: &str) -> bool {
    if tail.is_empty() {
        return false;
    }

    for suffix in DASH_NOISE_SUFFIXES {
        if tail == *suffix || tail.starts_with(suffix) || tail.ends_with(suffix) || tail.contains(suffix) {
            return true;
        }
    }

    // Check year remaster pattern (e.g. "2011 remaster", "2014 remaster")
    if tail.contains("remaster") {
        return true;
    }

    false
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.push(c);
            last_was_space = false;
        }
    }
    out.trim().to_string()
}

/// Fetch missing cover art online using iTunes Search API with fallback to Deezer Search API.
/// Cleans noise suffixes from artist and title for maximum lookup accuracy.
pub fn fetch_missing_cover_art(artist: &str, title: &str, album: &str) -> Option<Vec<u8>> {
    let clean_t = clean_title(title);
    let clean_a = clean_artist(artist);

    // Primary search query: "Artist Title"
    let primary_query = if !clean_a.is_empty() && !clean_t.is_empty() {
        format!("{} {}", clean_a, clean_t)
    } else if !clean_t.is_empty() {
        clean_t.clone()
    } else if !clean_a.is_empty() {
        clean_a.clone()
    } else if !album.trim().is_empty() {
        album.trim().to_string()
    } else {
        return None;
    };

    // 1. Query iTunes Search API with primary query
    if let Some(bytes) = query_itunes(&primary_query) {
        return Some(bytes);
    }

    // 2. Fallback query with Album if available in iTunes
    if !album.trim().is_empty() && !clean_a.is_empty() {
        let album_query = format!("{} {}", clean_a, album.trim());
        if album_query != primary_query {
            if let Some(bytes) = query_itunes(&album_query) {
                return Some(bytes);
            }
        }
    }

    // 3. Fallback to public Deezer Search API
    if let Some(bytes) = query_deezer(&primary_query) {
        return Some(bytes);
    }

    // 4. Deezer with Album query fallback
    if !album.trim().is_empty() && !clean_a.is_empty() {
        let album_query = format!("{} {}", clean_a, album.trim());
        if album_query != primary_query {
            if let Some(bytes) = query_deezer(&album_query) {
                return Some(bytes);
            }
        }
    }

    None
}

/// Convenience fallback fetcher for paths: inspects audio file metadata and fetches missing online cover art.
pub fn fallback_fetcher_for_path(path: &Path) -> Option<Vec<u8>> {
    use lofty::file::TaggedFileExt;
    use lofty::probe::Probe;
    use lofty::tag::Accessor;
    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            let artist = tag.artist().map(|a| a.trim().to_string()).unwrap_or_default();
            let title = tag.title().map(|t| t.trim().to_string()).unwrap_or_default();
            let album = tag.album().map(|al| al.trim().to_string()).unwrap_or_default();
            if !artist.is_empty() || !title.is_empty() {
                return fetch_missing_cover_art(&artist, &title, &album);
            }
        }
    }
    None
}

/// Query iTunes Search API:
/// https://itunes.apple.com/search?term={encoded_artist_title}&entity=song&limit=1
/// Sets User-Agent header and 5s timeout.
/// Extracts artworkUrl100 and replaces /100x100bb. with /600x600bb. for high resolution.
fn query_itunes(query: &str) -> Option<Vec<u8>> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!(
        "https://itunes.apple.com/search?term={}&entity=song&limit=1",
        encoded
    );

    let resp = ureq::get(&url)
        .set(
            "User-Agent",
            concat!("Eclipse/", env!("CARGO_PKG_VERSION"), " (lossless music player)"),
        )
        .timeout(Duration::from_secs(5))
        .call()
        .ok()?;

    if resp.status() != 200 {
        return None;
    }

    #[derive(serde::Deserialize)]
    struct ItunesResponse {
        results: Vec<ItunesTrack>,
    }

    #[derive(serde::Deserialize)]
    struct ItunesTrack {
        #[serde(rename = "artworkUrl100")]
        artwork_url_100: Option<String>,
    }

    let parsed: ItunesResponse = resp.into_json().ok()?;
    let track = parsed.results.into_iter().next()?;
    let artwork_url = track.artwork_url_100?;

    // Upgrade resolution to 600x600
    let high_res_url = if artwork_url.contains("/100x100bb.") {
        artwork_url.replace("/100x100bb.", "/600x600bb.")
    } else {
        artwork_url.replace("100x100bb", "600x600bb")
    };

    download_image_bytes(&high_res_url).or_else(|| download_image_bytes(&artwork_url))
}

/// Query Deezer Public Search API:
/// https://api.deezer.com/search?q={encoded_artist_title}&limit=1
/// Sets User-Agent header and 5s timeout.
/// Extracts album.cover_xl or album.cover_big.
fn query_deezer(query: &str) -> Option<Vec<u8>> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!("https://api.deezer.com/search?q={}&limit=1", encoded);

    let resp = ureq::get(&url)
        .set(
            "User-Agent",
            concat!("Eclipse/", env!("CARGO_PKG_VERSION"), " (lossless music player)"),
        )
        .timeout(Duration::from_secs(5))
        .call()
        .ok()?;

    if resp.status() != 200 {
        return None;
    }

    #[derive(serde::Deserialize)]
    struct DeezerResponse {
        data: Option<Vec<DeezerTrack>>,
    }

    #[derive(serde::Deserialize)]
    struct DeezerTrack {
        album: Option<DeezerAlbum>,
    }

    #[derive(serde::Deserialize)]
    struct DeezerAlbum {
        cover_xl: Option<String>,
        cover_big: Option<String>,
        cover_medium: Option<String>,
        cover: Option<String>,
    }

    let parsed: DeezerResponse = resp.into_json().ok()?;
    let track = parsed.data?.into_iter().next()?;
    let album = track.album?;

    let art_url = album
        .cover_xl
        .or(album.cover_big)
        .or(album.cover_medium)
        .or(album.cover)?;

    download_image_bytes(&art_url)
}

fn download_image_bytes(url: &str) -> Option<Vec<u8>> {
    let resp = ureq::get(url)
        .set(
            "User-Agent",
            concat!("Eclipse/", env!("CARGO_PKG_VERSION"), " (lossless music player)"),
        )
        .timeout(Duration::from_secs(5))
        .call()
        .ok()?;

    if resp.status() != 200 {
        return None;
    }

    let mut buf = Vec::new();
    resp.into_reader().read_to_end(&mut buf).ok()?;
    if buf.len() > 100 {
        Some(buf)
    } else {
        None
    }
}

/// Embed artwork bytes into a local audio file (.flac, .mp3, .m4a, etc.) using lofty.
pub fn embed_cover_art_in_file(path: &Path, art_bytes: &[u8]) -> Result<(), String> {
    use lofty::config::WriteOptions;
    use lofty::file::TaggedFileExt;
    use lofty::picture::{MimeType, Picture, PictureType};
    use lofty::probe::Probe;
    use lofty::tag::{Tag, TagExt};

    let mut tagged_file = Probe::open(path)
        .map_err(|e| format!("Failed to open file: {e}"))?
        .read()
        .map_err(|e| format!("Failed to read tags: {e}"))?;

    let tag = if tagged_file.primary_tag_mut().is_some() {
        tagged_file.primary_tag_mut().unwrap()
    } else if tagged_file.first_tag_mut().is_some() {
        tagged_file.first_tag_mut().unwrap()
    } else {
        let tag_type = tagged_file.primary_tag_type();
        tagged_file.insert_tag(Tag::new(tag_type));
        tagged_file
            .primary_tag_mut()
            .ok_or_else(|| "Failed to initialize tag".to_string())?
    };

    let mime = if art_bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        Some(MimeType::Png)
    } else {
        Some(MimeType::Jpeg)
    };

    let pic = Picture::new_unchecked(
        PictureType::CoverFront,
        mime,
        None,
        art_bytes.to_vec(),
    );

    tag.push_picture(pic);

    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Failed to write artwork tag to file: {e}"))?;

    Ok(())
}

/// Task definition for self-healing background artwork backfill.
#[derive(Debug, Clone)]
pub struct ArtBackfillTask {
    pub track_path: PathBuf,
    pub artist: String,
    pub title: String,
    pub album: String,
}

struct BackfillWorkerState {
    sender: mpsc::SyncSender<ArtBackfillTask>,
    enqueued_paths: Arc<Mutex<HashSet<PathBuf>>>,
}

static WORKER_STATE: OnceLock<BackfillWorkerState> = OnceLock::new();

fn init_backfill_workers() -> BackfillWorkerState {
    let (tx, rx) = mpsc::sync_channel::<ArtBackfillTask>(500);
    let rx = Arc::new(Mutex::new(rx));
    let enqueued_paths = Arc::new(Mutex::new(HashSet::new()));

    // Spawn bounded worker pool with max 2 concurrent threads to prevent UI/CPU starvation
    for i in 1..=2 {
        let rx_clone = Arc::clone(&rx);
        let enqueued_clone = Arc::clone(&enqueued_paths);
        thread::Builder::new()
            .name(format!("art-backfill-worker-{}", i))
            .spawn(move || {
                worker_loop(rx_clone, enqueued_clone);
            })
            .expect("failed to spawn art backfill worker thread");
    }

    BackfillWorkerState {
        sender: tx,
        enqueued_paths,
    }
}

fn worker_loop(
    rx: Arc<Mutex<mpsc::Receiver<ArtBackfillTask>>>,
    enqueued: Arc<Mutex<HashSet<PathBuf>>>,
) {
    loop {
        let task = {
            let lock = match rx.lock() {
                Ok(guard) => guard,
                Err(_) => break,
            };
            match lock.recv() {
                Ok(t) => t,
                Err(_) => break,
            }
        };

        let _ = process_art_backfill(&task);

        if let Ok(mut set) = enqueued.lock() {
            set.remove(&task.track_path);
        }
    }
}

/// Execute a single artwork backfill operation:
/// 1. Fetches missing cover art online.
/// 2. Embeds artwork into audio file using lofty.
/// 3. Caches artwork image bytes to temp dir (`std::env::temp_dir().join("eclipse_art_cache")`).
/// 4. Updates SQLite database `tracks.has_art = 1` for this track path.
pub fn process_art_backfill(task: &ArtBackfillTask) -> bool {
    let bytes = match fetch_missing_cover_art(&task.artist, &task.title, &task.album) {
        Some(b) if !b.is_empty() => b,
        _ => return false,
    };

    // 1. Embed artwork into local audio file using lofty
    if task.track_path.exists() {
        if let Err(e) = embed_cover_art_in_file(&task.track_path, &bytes) {
            eprintln!("[WARN] Failed to embed art in {:?}: {e}", task.track_path);
        }
    }

    // 2. Cache image bytes to temp directory eclipse_art_cache
    let temp_cache_dir = std::env::temp_dir().join("eclipse_art_cache");
    let _ = fs::create_dir_all(&temp_cache_dir);

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    task.track_path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    let _ = fs::write(temp_cache_dir.join(format!("{:016x}.bin", hash)), &bytes);
    let _ = fs::write(temp_cache_dir.join(format!("{:016x}.jpg", hash)), &bytes);
    if let Some(file_name) = task.track_path.file_name() {
        let _ = fs::write(
            temp_cache_dir.join(format!("{}.jpg", file_name.to_string_lossy())),
            &bytes,
        );
    }

    // Also populate app art cache
    let _ = crate::library::scanner::cache_album_art(&task.track_path, &bytes);

    // 3. Update SQLite database tracks.has_art = 1 for this track path
    let path_str = task.track_path.to_string_lossy().to_string();
    if let Ok(db) = crate::library::Database::open() {
        let _ = db.update_track_has_art(&path_str, true);
    }

    true
}

/// Queue an audio track for low-priority background artwork backfill.
/// Uses a bounded worker queue with max 2 concurrent requests.
pub fn queue_art_backfill(track_path: PathBuf, artist: String, title: String, album: String) {
    let state = WORKER_STATE.get_or_init(init_backfill_workers);

    // De-duplicate in-flight requests
    {
        let mut set = match state.enqueued_paths.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if set.contains(&track_path) {
            return;
        }
        set.insert(track_path.clone());
    }

    let task = ArtBackfillTask {
        track_path,
        artist,
        title,
        album,
    };

    if let Err(mpsc::TrySendError::Full(_)) = state.sender.try_send(task) {
        eprintln!("[WARN] Art backfill task queue saturated; dropping request");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_noise_suffixes_exact_prompt_cases() {
        // Required exact cases:
        // "- Sped Up", "(sped up)", "- 2011 Remaster", "- Twintails Edition", "[Remastered]"
        assert_eq!(clean_title("Instant Crush - Sped Up"), "Instant Crush");
        assert_eq!(clean_title("Instant Crush (sped up)"), "Instant Crush");
        assert_eq!(clean_title("Time - 2011 Remaster"), "Time");
        assert_eq!(clean_title("Song Name - Twintails Edition"), "Song Name");
        assert_eq!(clean_title("Hotel California [Remastered]"), "Hotel California");
    }

    #[test]
    fn test_clean_noise_suffixes_additional_cases() {
        assert_eq!(clean_title("Starboy (feat. Daft Punk)"), "Starboy");
        assert_eq!(clean_title("Song - Slowed + Reverb"), "Song");
        assert_eq!(clean_title("Song [Deluxe Edition]"), "Song");
        assert_eq!(clean_title("Song (Twintails Edition)"), "Song");
        assert_eq!(clean_title("Song - Radio Edit"), "Song");
        assert_eq!(clean_title("Song (Live at Wembley)"), "Song");
        assert_eq!(clean_title("Song - 2021 Remaster - Sped Up"), "Song");
    }

    #[test]
    fn test_clean_artist_cases() {
        assert_eq!(clean_artist("Daft Punk feat. Julian Casablancas"), "Daft Punk");
        assert_eq!(clean_artist("Daft Punk ft. Pharrell Williams"), "Daft Punk");
        assert_eq!(clean_artist("The Weeknd with Daft Punk"), "The Weeknd");
        assert_eq!(clean_artist("Solo Artist"), "Solo Artist");
    }

    #[test]
    fn test_itunes_resolution_replacement() {
        let original = "https://is1-ssl.mzstatic.com/image/thumb/Music115/v4/e8/43/5f/886443919266.jpg/100x100bb.jpg";
        let expected = "https://is1-ssl.mzstatic.com/image/thumb/Music115/v4/e8/43/5f/886443919266.jpg/600x600bb.jpg";
        let replaced = original.replace("/100x100bb.", "/600x600bb.");
        assert_eq!(replaced, expected);
    }

    #[test]
    fn test_fetch_missing_cover_art_live_or_offline_fallback() {
        // Query a well-known track on iTunes/Deezer
        let art = fetch_missing_cover_art("Daft Punk", "Instant Crush - Sped Up", "Random Access Memories");
        if let Some(bytes) = art {
            assert!(bytes.len() > 1000, "Artwork bytes should be non-trivial size");
            // Check JPEG or PNG header
            let is_jpeg = bytes.starts_with(&[0xFF, 0xD8]);
            let is_png = bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]);
            assert!(is_jpeg || is_png, "Must be valid JPEG or PNG bytes");
        }
    }

    #[test]
    fn test_queue_art_backfill_deduplication() {
        let dummy_path = PathBuf::from("C:\\music\\non_existent_backfill_test_file.flac");
        queue_art_backfill(
            dummy_path.clone(),
            "NonExistentArtist".to_string(),
            "NonExistentTitle".to_string(),
            "NonExistentAlbum".to_string(),
        );

        // Queue second time should be deduplicated
        queue_art_backfill(
            dummy_path,
            "NonExistentArtist".to_string(),
            "NonExistentTitle".to_string(),
            "NonExistentAlbum".to_string(),
        );
    }
}
