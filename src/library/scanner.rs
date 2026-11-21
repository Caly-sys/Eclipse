use crate::config::app_data_dir;
use crate::library::db::Database;
use crate::library::model::Track;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::flac::FlacFile;
use lofty::ogg::OggPictureStorage;
use lofty::picture::PictureType;
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const FOLDER_ART_FILENAMES: &[&str] = &[
    "cover.jpg",
    "cover.jpeg",
    "cover.png",
    "folder.jpg",
    "folder.jpeg",
    "folder.png",
    "front.jpg",
    "front.png",
    "album.jpg",
    "album.png",
];

/// Hook type for external/online artwork fallback fetchers (e.g. crate::library::art_fetcher::fetch_missing_cover_art).
pub type FallbackArtFetcher = fn(&Path) -> Option<Vec<u8>>;
static FALLBACK_FETCHER: std::sync::RwLock<Option<FallbackArtFetcher>> = std::sync::RwLock::new(None);

/// Register an external/online artwork fallback fetcher hook.
pub fn set_fallback_art_fetcher(fetcher: FallbackArtFetcher) {
    if let Ok(mut lock) = FALLBACK_FETCHER.write() {
        *lock = Some(fetcher);
    }
}

pub fn get_fallback_art(path: &Path) -> Option<Vec<u8>> {
    if let Ok(lock) = FALLBACK_FETCHER.read() {
        if let Some(f) = *lock {
            return f(path);
        }
    }
    None
}

/// Search directory containing the audio file for local cover art files.
pub fn find_folder_artwork(folder: &Path) -> Option<Vec<u8>> {
    for name in FOLDER_ART_FILENAMES {
        let art_path = folder.join(name);
        if art_path.is_file() {
            if let Ok(bytes) = fs::read(&art_path) {
                if !bytes.is_empty() {
                    return Some(bytes);
                }
            }
        }
    }
    None
}

/// Extract embedded cover art from an already-read TaggedFile.
/// Searches all tags for CoverFront first, falling back to any picture type.
pub fn extract_artwork_from_tagged_file(tagged_file: &lofty::file::TaggedFile) -> Option<Vec<u8>> {
    // 1. Search for PictureType::CoverFront across all tags
    for tag in tagged_file.tags() {
        for pic in tag.pictures() {
            if pic.pic_type() == PictureType::CoverFront {
                return Some(pic.data().to_vec());
            }
        }
    }

    // 2. Fall back to the first available picture of ANY type
    for tag in tagged_file.tags() {
        if let Some(pic) = tag.pictures().first() {
            return Some(pic.data().to_vec());
        }
    }

    None
}

/// Extract cover artwork from an audio file or its containing folder.
/// 1. Probes all Lofty tags for CoverFront, then any picture type.
/// 2. For FLAC files, checks native FLAC picture blocks.
/// 3. Falls back to folder artwork (cover.jpg, folder.jpg, front.png, etc.).
pub fn extract_artwork_from_file(path: &Path) -> Option<Vec<u8>> {
    // 1. Try lofty tagged file across all tags
    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
        if let Some(art) = extract_artwork_from_tagged_file(&tagged_file) {
            return Some(art);
        }
    }

    // 2. Extra FLAC native picture block inspection
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("flac") {
            if let Ok(mut f) = fs::File::open(path) {
                if let Ok(flac_file) = FlacFile::read_from(&mut f, lofty::config::ParseOptions::new()) {
                    for (pic, _) in flac_file.pictures() {
                        if pic.pic_type() == PictureType::CoverFront {
                            return Some(pic.data().to_vec());
                        }
                    }
                    if let Some((pic, _)) = flac_file.pictures().first() {
                        return Some(pic.data().to_vec());
                    }
                }
            }
        }
    }

    // 3. Fast folder artwork fallback
    if let Some(parent) = path.parent() {
        if let Some(bytes) = find_folder_artwork(parent) {
            return Some(bytes);
        }
    }

    None
}

const AUDIO_EXTENSIONS: &[&str] = &[
    "flac", "mp3", "wav", "ogg", "m4a", "aac", "opus", "aiff", "wv",
];

pub fn scan_directories(folders: &[String], db: &Database) -> (usize, usize) {
    let mut total_scanned = 0;
    let mut added_or_updated = 0;
    let mut batch = Vec::with_capacity(150);
    let mut missing_genres_found = false;

    for folder in folders {
        let p = Path::new(folder);
        if !p.exists() || !p.is_dir() {
            continue;
        }

        for entry in WalkDir::new(p).follow_links(true).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if AUDIO_EXTENSIONS.contains(&ext_lower.as_str()) {
                        total_scanned += 1;
                        if let Some(track) = inspect_audio_file(path) {
                            if crate::library::genre_fetcher::is_missing_or_unknown_genre(&track.genre) {
                                missing_genres_found = true;
                            }
                            batch.push(track);
                            if batch.len() >= 150 {
                                if let Ok(n) = db.upsert_tracks_batch(&batch) {
                                    added_or_updated += n;
                                }
                                batch.clear();
                            }
                        }
                    }
                }
            }
        }
    }

    if !batch.is_empty() {
        if let Ok(n) = db.upsert_tracks_batch(&batch) {
            added_or_updated += n;
        }
        batch.clear();
    }

    // Trigger self-healing genre resolution for any discovered tracks with missing genres
    if missing_genres_found {
        crate::library::genre_fetcher::start_genre_backfill(std::sync::Arc::new(db.clone()));
    }

    // Automatically prune missing local tracks for the scanned library roots
    let configured_roots: Vec<PathBuf> = folders.iter().map(PathBuf::from).collect();
    let _ = db.prune_missing_local_tracks(&configured_roots);

    (total_scanned, added_or_updated)
}

pub fn inspect_audio_file(path: &Path) -> Option<Track> {
    let metadata = fs::metadata(path).ok()?;
    let file_size = metadata.len();
    let folder = path.parent().unwrap_or(Path::new("")).to_string_lossy().to_string();
    let format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_uppercase();

    let mut title = String::new();
    let mut artist = String::new();
    let mut album_artist = String::new();
    let mut album = String::new();
    let mut genre = String::new();
    let mut year = 0;
    let mut track_number = 0;
    let mut duration_secs = 0.0;
    let mut bitrate_kbps = 0;
    let mut sample_rate = 0;
    let mut channels = 2;
    let mut has_art = false;

    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
        let props = tagged_file.properties();
        duration_secs = props.duration().as_secs_f64();
        bitrate_kbps = props.audio_bitrate().unwrap_or(0);
        sample_rate = props.sample_rate().unwrap_or(0);
        channels = props.channels().unwrap_or(2) as u16;

        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            if let Some(t) = tag.title() {
                title = t.trim().to_string();
            }
            if let Some(a) = tag.artist() {
                artist = a.trim().to_string();
            }
            if let Some(al) = tag.album() {
                album = al.trim().to_string();
            }
            if let Some(g) = tag.genre() {
                genre = g.trim().to_string();
            }
            if let Some(y) = tag.year() {
                year = y;
            }
            if let Some(tn) = tag.track() {
                track_number = tn;
            }

            if let Some(item) = tag.get(&ItemKey::AlbumArtist) {
                if let Some(val) = item.value().text() {
                    album_artist = val.trim().to_string();
                }
            }
        }

        // Check embedded picture across all tags
        if let Some(art_bytes) = extract_artwork_from_tagged_file(&tagged_file) {
            has_art = true;
            let _ = cache_album_art(path, &art_bytes);
        }
    }

    // Extra check for FLAC native picture blocks if still no art
    if !has_art {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.eq_ignore_ascii_case("flac") {
                if let Ok(mut f) = fs::File::open(path) {
                    if let Ok(flac_file) = FlacFile::read_from(&mut f, lofty::config::ParseOptions::new()) {
                        for (pic, _) in flac_file.pictures() {
                            if pic.pic_type() == PictureType::CoverFront {
                                has_art = true;
                                let _ = cache_album_art(path, pic.data());
                                break;
                            }
                        }
                        if !has_art {
                            if let Some((pic, _)) = flac_file.pictures().first() {
                                has_art = true;
                                let _ = cache_album_art(path, pic.data());
                            }
                        }
                    }
                }
            }
        }
    }

    // If no embedded art, check for local cover.jpg / folder.jpg in the directory
    if !has_art {
        if let Some(parent) = path.parent() {
            if let Some(bytes) = find_folder_artwork(parent) {
                has_art = true;
                let _ = cache_album_art(path, &bytes);
            }
        }
    }

    // Fallback title from filename
    if title.is_empty() {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            title = stem.to_string();
        }
    }

    Some(Track {
        id: 0,
        path: path.to_string_lossy().to_string(),
        title,
        artist,
        album_artist,
        album,
        genre,
        year,
        track_number,
        duration_secs,
        bitrate_kbps,
        sample_rate,
        channels,
        format,
        file_size,
        play_count: 0,
        favorite: false,
        has_art,
        folder,
    })
}

pub fn art_cache_dir() -> PathBuf {
    let d = app_data_dir().join("art_cache");
    let _ = fs::create_dir_all(&d);
    d
}

pub fn art_cache_path(track_path: &Path) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    track_path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();
    art_cache_dir().join(format!("{:016x}.bin", hash))
}

pub fn cache_album_art(track_path: &Path, data: &[u8]) -> std::io::Result<PathBuf> {
    let out = art_cache_path(track_path);
    fs::write(&out, data)?;
    Ok(out)
}

pub fn load_cached_album_art(track_path: &Path) -> Option<Vec<u8>> {
    let path = art_cache_path(track_path);
    if path.exists() {
        fs::read(path).ok()
    } else {
        None
    }
}

fn extract_tags_for_fetch(path: &Path) -> (String, String, String) {
    let mut title = String::new();
    let mut artist = String::new();
    let mut album = String::new();

    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            if let Some(t) = tag.title() {
                title = t.trim().to_string();
            }
            if let Some(a) = tag.artist() {
                artist = a.trim().to_string();
            }
            if let Some(al) = tag.album() {
                album = al.trim().to_string();
            }
        }
    }

    if title.is_empty() {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            title = stem.to_string();
        }
    }

    (artist, title, album)
}

pub fn get_or_extract_cover_art(track_path: &Path) -> Option<Vec<u8>> {
    // 1. Try loading from disk cache first
    if let Some(bytes) = load_cached_album_art(track_path) {
        return Some(bytes);
    }

    // 2. Try extracting from audio file (all lofty tags, FLAC picture blocks, or folder fallback)
    if let Some(bytes) = extract_artwork_from_file(track_path) {
        let _ = cache_album_art(track_path, &bytes);
        return Some(bytes);
    }

    // 3. Fallback hook (e.g. registered external fetcher)
    if let Some(bytes) = get_fallback_art(track_path) {
        let _ = cache_album_art(track_path, &bytes);
        return Some(bytes);
    }

    // 4. Online fallback via crate::library::art_fetcher before giving up
    let (artist, title, album) = extract_tags_for_fetch(track_path);
    if (!artist.is_empty() && !title.is_empty()) || !album.is_empty() {
        if let Some(bytes) = crate::library::art_fetcher::fetch_missing_cover_art(&artist, &title, &album) {
            let _ = cache_album_art(track_path, &bytes);
            return Some(bytes);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_folder_artwork() {
        let temp_dir = std::env::temp_dir().join("eclipse_test_folder_art");
        let _ = fs::create_dir_all(&temp_dir);
        let cover_path = temp_dir.join("cover.jpg");
        fs::write(&cover_path, b"fake_jpeg_data").unwrap();

        let found = find_folder_artwork(&temp_dir);
        assert_eq!(found, Some(b"fake_jpeg_data".to_vec()));

        let _ = fs::remove_file(cover_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_fallback_art_fetcher_hook() {
        fn mock_fetcher(_path: &Path) -> Option<Vec<u8>> {
            Some(b"mock_online_art".to_vec())
        }

        set_fallback_art_fetcher(mock_fetcher);
        let dummy_path = Path::new("non_existent_unique_track_9999.flac");
        let art = get_or_extract_cover_art(dummy_path);
        assert_eq!(art, Some(b"mock_online_art".to_vec()));

        // Should now be cached
        let cached = load_cached_album_art(dummy_path);
        assert_eq!(cached, Some(b"mock_online_art".to_vec()));

        // Clean up cache file
        let cache_file = art_cache_path(dummy_path);
        let _ = fs::remove_file(cache_file);
    }
}
