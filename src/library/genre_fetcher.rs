use crate::library::db::LibraryDatabase;
use lofty::config::WriteOptions;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::{Accessor, Tag, TagExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static GENRE_FETCH_RUNNING: AtomicBool = AtomicBool::new(false);
static GENRE_FETCH_PROGRESS: Mutex<Option<(usize, usize)>> = Mutex::new(None);

/// Check if a genre string is considered missing, empty, or unknown.
pub fn is_missing_or_unknown_genre(genre: &str) -> bool {
    let trimmed = genre.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("unknown")
        || trimmed.eq_ignore_ascii_case("unknown genre")
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("other")
        || trimmed.eq_ignore_ascii_case("undefined")
}

/// Clean noise tokens from track title for genre search accuracy.
pub fn clean_title(title: &str) -> String {
    crate::library::art_fetcher::clean_title(title)
}

/// Clean featuring tokens from artist name for genre search accuracy.
pub fn clean_artist(artist: &str) -> String {
    crate::library::art_fetcher::clean_artist(artist)
}

/// Fetch primary genre for a track using iTunes Search API with Deezer fallback.
pub fn fetch_track_genre(artist: &str, title: &str) -> Option<String> {
    fetch_track_genre_with_album(artist, title, "")
}

/// Fetch primary genre for a track with optional album context.
pub fn fetch_track_genre_with_album(artist: &str, title: &str, album: &str) -> Option<String> {
    let clean_t = clean_title(title);
    let clean_a = clean_artist(artist);

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

    // 1. Primary search with iTunes Search API
    if let Some(genre) = query_itunes_genre(&primary_query) {
        return Some(genre);
    }

    // 2. Fallback query with Album if provided
    if !album.trim().is_empty() && !clean_a.is_empty() {
        let album_query = format!("{} {}", clean_a, album.trim());
        if album_query != primary_query {
            if let Some(genre) = query_itunes_genre(&album_query) {
                return Some(genre);
            }
        }
    }

    // 3. Fallback to Deezer Public Search API
    if let Some(genre) = query_deezer_genre(&primary_query) {
        return Some(genre);
    }

    // 4. Deezer with Album query fallback
    if !album.trim().is_empty() && !clean_a.is_empty() {
        let album_query = format!("{} {}", clean_a, album.trim());
        if album_query != primary_query {
            if let Some(genre) = query_deezer_genre(&album_query) {
                return Some(genre);
            }
        }
    }

    None
}

/// Query iTunes Search API:
/// https://itunes.apple.com/search?term={encoded}&entity=song&limit=1
fn query_itunes_genre(query: &str) -> Option<String> {
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
        #[serde(rename = "primaryGenreName")]
        primary_genre_name: Option<String>,
    }

    let parsed: ItunesResponse = resp.into_json().ok()?;
    let track = parsed.results.into_iter().next()?;
    let genre = track.primary_genre_name?;
    let trimmed = genre.trim();
    if !trimmed.is_empty() && !is_missing_or_unknown_genre(trimmed) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Query Deezer Public Search API with Album detail fallback:
/// https://api.deezer.com/search?q={encoded}&limit=1 -> album.id -> https://api.deezer.com/album/{id}
fn query_deezer_genre(query: &str) -> Option<String> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let search_url = format!("https://api.deezer.com/search?q={}&limit=1", encoded);

    let resp = ureq::get(&search_url)
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
    struct DeezerSearchResponse {
        data: Option<Vec<DeezerTrack>>,
    }

    #[derive(serde::Deserialize)]
    struct DeezerTrack {
        album: Option<DeezerAlbum>,
    }

    #[derive(serde::Deserialize)]
    struct DeezerAlbum {
        id: Option<i64>,
    }

    let parsed: DeezerSearchResponse = resp.into_json().ok()?;
    let album_id = parsed.data?.into_iter().next()?.album?.id?;

    let album_url = format!("https://api.deezer.com/album/{}", album_id);
    let album_resp = ureq::get(&album_url)
        .set(
            "User-Agent",
            concat!("Eclipse/", env!("CARGO_PKG_VERSION"), " (lossless music player)"),
        )
        .timeout(Duration::from_secs(5))
        .call()
        .ok()?;

    if album_resp.status() != 200 {
        return None;
    }

    #[derive(serde::Deserialize)]
    struct DeezerGenreItem {
        name: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct DeezerGenres {
        data: Option<Vec<DeezerGenreItem>>,
    }

    #[derive(serde::Deserialize)]
    struct DeezerAlbumDetail {
        genres: Option<DeezerGenres>,
    }

    let album_detail: DeezerAlbumDetail = album_resp.into_json().ok()?;
    let genre_name = album_detail.genres?.data?.into_iter().next()?.name?;
    let trimmed = genre_name.trim();
    if !trimmed.is_empty() && !is_missing_or_unknown_genre(trimmed) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Embed genre tag directly into a local audio file using Lofty.
pub fn embed_genre_in_file(path: &Path, genre: &str) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("File does not exist: {}", path.display()));
    }

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

    let trimmed = genre.trim();
    if !trimmed.is_empty() {
        tag.set_genre(trimmed.to_string());
    }

    tag.save_to_path(path, WriteOptions::default())
        .map_err(|e| format!("Failed to save genre tag to file: {e}"))?;

    Ok(())
}

/// Returns true if a background genre fetch or backfill process is currently active.
pub fn is_genre_fetch_running() -> bool {
    GENRE_FETCH_RUNNING.load(Ordering::SeqCst)
}

/// Returns current progress as Some((processed, total)) when active, or None when idle.
pub fn get_genre_fetch_progress() -> Option<(usize, usize)> {
    GENRE_FETCH_PROGRESS.lock().ok().and_then(|g| *g)
}

/// Start an asynchronous background backfill for all tracks in SQLite that lack a valid genre.
pub fn start_genre_backfill(db: Arc<LibraryDatabase>) {
    if GENRE_FETCH_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    thread::spawn(move || {
        struct CleanupGuard;
        impl Drop for CleanupGuard {
            fn drop(&mut self) {
                GENRE_FETCH_RUNNING.store(false, Ordering::SeqCst);
                if let Ok(mut lock) = GENRE_FETCH_PROGRESS.lock() {
                    *lock = None;
                }
            }
        }
        let _guard = CleanupGuard;

        let tracks = match db.get_tracks_with_missing_genre() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[genre_fetcher] Failed to query tracks with missing genre: {e}");
                return;
            }
        };

        let total = tracks.len();
        if total == 0 {
            return;
        }

        if let Ok(mut lock) = GENRE_FETCH_PROGRESS.lock() {
            *lock = Some((0, total));
        }

        let mut processed = 0;
        for track in tracks {
            if !track.artist.trim().is_empty() || !track.title.trim().is_empty() {
                if let Some(genre) =
                    fetch_track_genre_with_album(&track.artist, &track.title, &track.album)
                {
                    let trimmed = genre.trim();
                    if !trimmed.is_empty() {
                        if let Err(e) = db.update_track_genre(track.id, trimmed) {
                            eprintln!(
                                "[genre_fetcher] Failed to update track genre in DB for track {}: {e}",
                                track.id
                            );
                        }
                        let path = Path::new(&track.path);
                        if path.is_file() {
                            let _ = embed_genre_in_file(path, trimmed);
                        }
                    }
                }
            }

            processed += 1;
            if let Ok(mut lock) = GENRE_FETCH_PROGRESS.lock() {
                *lock = Some((processed, total));
            }

            // Polite delay between network requests to respect public API rate limits
            thread::sleep(Duration::from_millis(150));
        }
    });
}

/// Queue an individual track for asynchronous background genre resolution and file self-healing.
pub fn queue_genre_backfill(
    db: Arc<LibraryDatabase>,
    track_id: i64,
    path: PathBuf,
    artist: String,
    title: String,
) {
    thread::spawn(move || {
        if let Some(genre) = fetch_track_genre(&artist, &title) {
            let trimmed = genre.trim();
            if !trimmed.is_empty() {
                let _ = db.update_track_genre(track_id, trimmed);
                if path.is_file() {
                    let _ = embed_genre_in_file(&path, trimmed);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_noise_tokens() {
        assert_eq!(clean_title("Numb - Sped Up"), "Numb");
        assert_eq!(clean_title("In the End (Remix)"), "In the End");
        assert_eq!(
            clean_title("Comfortably Numb [2011 Remaster]"),
            "Comfortably Numb"
        );
        assert_eq!(clean_title("Starboy - Slowed + Reverb"), "Starboy");
        assert_eq!(clean_artist("The Weeknd feat. Daft Punk"), "The Weeknd");
    }

    #[test]
    fn test_is_missing_or_unknown_genre() {
        assert!(is_missing_or_unknown_genre(""));
        assert!(is_missing_or_unknown_genre("   "));
        assert!(is_missing_or_unknown_genre("Unknown"));
        assert!(is_missing_or_unknown_genre("unknown genre"));
        assert!(is_missing_or_unknown_genre("None"));
        assert!(is_missing_or_unknown_genre("Other"));
        assert!(!is_missing_or_unknown_genre("Rock"));
        assert!(!is_missing_or_unknown_genre("J-Pop"));
        assert!(!is_missing_or_unknown_genre("Electronic"));
    }

    #[test]
    fn test_fetch_track_genre_live_or_offline() {
        let genre = fetch_track_genre("Coldplay", "Yellow");
        if let Some(g) = genre {
            assert!(!g.is_empty());
            assert!(!is_missing_or_unknown_genre(&g));
        }
    }
}
