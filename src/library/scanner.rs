use crate::config::app_data_dir;
use crate::library::db::Database;
use crate::library::model::Track;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &[
    "flac", "mp3", "wav", "ogg", "m4a", "aac", "opus", "aiff", "wv",
];

pub fn scan_directories(folders: &[String], db: &Database) -> (usize, usize) {
    let mut total_scanned = 0;
    let mut added_or_updated = 0;

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
                            if db.upsert_track(&track).is_ok() {
                                added_or_updated += 1;
                            }
                        }
                    }
                }
            }
        }
    }

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

            // Check embedded picture
            let pictures = tag.pictures();
            if !pictures.is_empty() {
                has_art = true;
                let _ = cache_album_art(path, pictures[0].data());
            }
        }
    }

    // If no embedded art, check for local cover.jpg / folder.jpg in the directory
    if !has_art {
        if let Some(parent) = path.parent() {
            for name in &["cover.jpg", "folder.jpg", "cover.png", "folder.png", "front.jpg"] {
                let art_path = parent.join(name);
                if art_path.exists() {
                    has_art = true;
                    if let Ok(bytes) = fs::read(&art_path) {
                        let _ = cache_album_art(path, &bytes);
                    }
                    break;
                }
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

pub fn get_or_extract_cover_art(track_path: &Path) -> Option<Vec<u8>> {
    // 1. Try loading from disk cache first
    if let Some(bytes) = load_cached_album_art(track_path) {
        return Some(bytes);
    }

    // 2. Try extracting from audio file tags via lofty
    if let Ok(tagged_file) = Probe::open(track_path).and_then(|p| p.read()) {
        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            let pictures = tag.pictures();
            if !pictures.is_empty() {
                let data = pictures[0].data();
                let _ = cache_album_art(track_path, data);
                return Some(data.to_vec());
            }
        }
    }

    // 3. Try finding cover.jpg / folder.jpg / front.jpg in the track's folder
    if let Some(parent) = track_path.parent() {
        for name in &["cover.jpg", "folder.jpg", "cover.png", "folder.png", "front.jpg", "album.jpg"] {
            let art_file = parent.join(name);
            if art_file.exists() {
                if let Ok(data) = fs::read(&art_file) {
                    let _ = cache_album_art(track_path, &data);
                    return Some(data);
                }
            }
        }
    }

    None
}
