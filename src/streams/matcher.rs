use crate::library::db::Database;
use crate::library::model::Track;
use crate::streams::provider::StreamTrack;

pub const DEFAULT_DURATION_TOLERANCE_SECS: f64 = 5.0;

/// Normalizes a string for audio library matching:
/// 1. Lowercase and fullwidth ASCII/whitespace mapping.
/// 2. CJK-safe: preserves Japanese (Kanji, Hiragana, Katakana), Chinese (Hanzi), and Korean (Hangul).
/// 3. Noise token stripping: `(feat. ...)`, `[feat. ...]`, `ft. ...`, `(remix)`, `[remastered ...]`, `(live ...)`.
/// 4. Punctuation removal for ASCII and halfwidth/fullwidth punctuation.
/// 5. Whitespace collapsing and trimming.
pub fn normalize_string(s: &str) -> String {
    let stripped = {
        let s_clean = strip_noise_tokens(s);
        if s_clean.trim().is_empty() {
            s.to_string()
        } else {
            s_clean
        }
    };

    let mut out = String::with_capacity(stripped.len());

    for c in stripped.chars() {
        // Map fullwidth Latin & digits to standard ASCII; map ideographic space to ASCII space
        let norm_char = match c {
            '\u{3000}' => ' ',
            '\u{FF01}'..='\u{FF5E}' => {
                let code = (c as u32) - 0xFEE0;
                char::from_u32(code).unwrap_or(c)
            }
            _ => c,
        };

        if norm_char.is_alphanumeric() {
            // Preserves ASCII alphanumeric + CJK (Kanji, Hiragana, Katakana, Hangul) + accented chars
            for lc in norm_char.to_lowercase() {
                out.push(lc);
            }
        } else if norm_char.is_whitespace() {
            out.push(' ');
        } else if matches!(norm_char, '-' | '/' | '_' | '\\' | '|') {
            // Replace separators with spaces to prevent word concatenation (e.g. "hip-hop" -> "hip hop")
            out.push(' ');
        }
        // Other punctuation (ASCII and CJK punctuation like 、 。) is safely omitted
    }

    // Collapse multiple consecutive spaces and trim
    let mut collapsed = String::with_capacity(out.len());
    let mut last_was_space = false;
    for c in out.chars() {
        if c == ' ' {
            if !last_was_space {
                collapsed.push(' ');
                last_was_space = true;
            }
        } else {
            collapsed.push(c);
            last_was_space = false;
        }
    }

    collapsed.trim().to_string()
}

/// Strips bracketed and unbracketed noise tokens from titles and artist strings.
fn strip_noise_tokens(s: &str) -> String {
    let mut cleaned = String::with_capacity(s.len());
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

            if found_close && is_bracket_noise_token(&inner) {
                // Stripped noise token; keep a space separator
                cleaned.push(' ');
            } else {
                cleaned.push(c);
                cleaned.push_str(&inner);
                if found_close {
                    cleaned.push(close);
                }
            }
        } else {
            cleaned.push(c);
        }
    }

    let unbracketed = strip_unbracketed_noise(&cleaned);
    if !unbracketed.trim().is_empty() {
        unbracketed
    } else {
        cleaned
    }
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

/// Checks if inner content enclosed in brackets is a common audio noise tag.
fn is_bracket_noise_token(inner: &str) -> bool {
    let lower = inner.trim().to_lowercase();
    if lower.is_empty() {
        return false;
    }

    // feat. / ft. / featuring
    if lower.starts_with("feat.")
        || lower.starts_with("feat ")
        || lower.starts_with("featuring")
        || lower.starts_with("ft.")
        || lower.starts_with("ft ")
        || lower.starts_with("with ")
    {
        return true;
    }

    // remix
    if lower == "remix"
        || lower.ends_with(" remix")
        || lower.starts_with("remix ")
        || lower.contains("remix")
    {
        return true;
    }

    // remaster
    if lower.contains("remaster") {
        return true;
    }

    // live
    if lower == "live"
        || lower.starts_with("live ")
        || lower.starts_with("live at")
        || lower.starts_with("live in")
        || lower.starts_with("live from")
        || lower.ends_with(" live")
    {
        return true;
    }

    // standard edition tags
    if lower.contains("bonus track")
        || lower.contains("deluxe")
        || lower.contains("radio edit")
        || lower.contains("instrumental")
        || lower.contains("acoustic")
        || lower.contains("version")
    {
        return true;
    }

    false
}

/// Strips unbracketed noise tokens such as `ft. ...` or trailing ` - Remastered`.
fn strip_unbracketed_noise(s: &str) -> String {
    let lower = s.to_lowercase();

    // Check for featuring tokens
    let feat_patterns = [" ft. ", " feat. ", " featuring ", " ft ", " feat "];
    let mut earliest_feat: Option<usize> = None;
    for pat in feat_patterns {
        if let Some(pos) = lower.find(pat) {
            earliest_feat = match earliest_feat {
                Some(prev) => Some(prev.min(pos)),
                None => Some(pos),
            };
        }
    }

    let mut result = if let Some(pos) = earliest_feat {
        let prefix = &s[..pos];
        if !prefix.trim().is_empty() {
            prefix.to_string()
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    };

    // Check for trailing hyphen noise (e.g. " - Remastered", " - 2021 Remaster", " - Live", " - Remix")
    for sep in [" - ", " – ", " — "] {
        if let Some(pos) = result.rfind(sep) {
            let tail = result[pos + sep.len()..].trim().to_lowercase();
            if tail == "remix"
                || tail.ends_with("remix")
                || tail.contains("remaster")
                || tail == "live"
                || tail.starts_with("live ")
            {
                let prefix = &result[..pos];
                if !prefix.trim().is_empty() {
                    result = prefix.to_string();
                }
            }
        }
    }

    result
}

/// Extracts primary artist from an artist string (before commas, ampersands, or featuring tokens).
pub fn primary_artist(artist: &str) -> String {
    let lower = artist.to_lowercase();
    let separators = [
        " feat. ", " feat ", " ft. ", " ft ", " featuring ", " & ", " / ", " x ", " and ", ",",
        "、",
    ];

    let mut earliest: Option<usize> = None;
    for sep in separators {
        if let Some(pos) = lower.find(sep) {
            earliest = match earliest {
                Some(prev) => Some(prev.min(pos)),
                None => Some(pos),
            };
        }
    }

    if let Some(pos) = earliest {
        let first = &artist[..pos];
        if !first.trim().is_empty() {
            return normalize_string(first);
        }
    }

    normalize_string(artist)
}

/// Matches a single stream track against candidate local tracks.
/// Returns the matched local track reference and confidence score if a match is found.
pub fn match_single_track<'a>(
    stream_track: &StreamTrack,
    candidates: &'a [Track],
    duration_tolerance_secs: f64,
) -> Option<(&'a Track, f64)> {
    let s_title = normalize_string(&stream_track.title);
    let s_artist = normalize_string(&stream_track.artist);
    let s_primary_artist = primary_artist(&stream_track.artist);
    let s_album = normalize_string(&stream_track.album);

    if s_title.is_empty() {
        return None;
    }

    let mut best_match: Option<(&'a Track, f64)> = None;

    for track in candidates {
        // Duration filter: if both durations are available and > 0, enforce strict tolerance
        let duration_diff = if stream_track.duration_secs > 0.0 && track.duration_secs > 0.0 {
            let diff = (stream_track.duration_secs - track.duration_secs).abs();
            if diff > duration_tolerance_secs {
                continue;
            }
            diff
        } else {
            0.0
        };

        // Title matching: exact match or containment
        let l_title = normalize_string(track.display_title());
        if l_title.is_empty() {
            continue;
        }

        let title_score = if s_title == l_title {
            100.0
        } else if s_title.contains(&l_title) || l_title.contains(&s_title) {
            let min_len = s_title.len().min(l_title.len());
            if min_len >= 3 {
                75.0
            } else {
                continue;
            }
        } else {
            continue;
        };

        // Artist matching: exact, primary artist, or containment
        let l_artist = normalize_string(track.display_artist());
        let l_album_artist = normalize_string(&track.album_artist);
        let l_primary_artist = primary_artist(track.display_artist());

        let mut artist_score = 0.0;
        if !s_artist.is_empty()
            && (s_artist == l_artist
                || (!l_album_artist.is_empty() && s_artist == l_album_artist))
        {
            artist_score = 100.0;
        } else if (!s_primary_artist.is_empty()
            && (s_primary_artist == l_primary_artist || s_primary_artist == l_artist))
            || (!l_primary_artist.is_empty() && l_primary_artist == s_artist)
        {
            artist_score = 85.0;
        } else if (!s_artist.is_empty()
            && !l_artist.is_empty()
            && (s_artist.contains(&l_artist) || l_artist.contains(&s_artist)))
            || (!s_primary_artist.is_empty()
                && !l_artist.is_empty()
                && (l_artist.contains(&s_primary_artist) || s_artist.contains(&l_primary_artist)))
            || (!s_artist.is_empty()
                && !l_album_artist.is_empty()
                && (s_artist.contains(&l_album_artist) || l_album_artist.contains(&s_artist)))
        {
            artist_score = 70.0;
        }

        // If artist fails to match and stream artist was specified, reject candidate
        if artist_score < 70.0 && !s_artist.is_empty() {
            continue;
        }

        // Album match bonus
        let mut album_bonus = 0.0;
        let l_album = normalize_string(track.display_album());
        if !s_album.is_empty() && !l_album.is_empty() {
            if s_album == l_album {
                album_bonus = 15.0;
            } else if s_album.contains(&l_album) || l_album.contains(&s_album) {
                album_bonus = 10.0;
            }
        }

        // Duration closeness bonus
        let dur_bonus = if stream_track.duration_secs > 0.0 && track.duration_secs > 0.0 {
            ((duration_tolerance_secs - duration_diff).max(0.0) / duration_tolerance_secs) * 15.0
        } else {
            0.0
        };

        // Lossless audio preference
        let lossless_bonus = if track.is_lossless() { 5.0 } else { 0.0 };

        let total_score = title_score + artist_score + album_bonus + dur_bonus + lossless_bonus;

        if total_score >= 135.0 {
            match &best_match {
                Some((_, prev_score)) if *prev_score >= total_score => {}
                _ => {
                    best_match = Some((track, total_score));
                }
            }
        }
    }

    best_match
}

/// Matches a list of stream tracks against candidate local tracks using the default 5.0s duration tolerance.
/// Returns a new `Vec<StreamTrack>` with `local_track_id` and `local_path` populated for matches.
pub fn match_tracks_against_library(
    stream_tracks: &[StreamTrack],
    library_tracks: &[Track],
) -> Vec<StreamTrack> {
    match_tracks_against_library_with_tolerance(
        stream_tracks,
        library_tracks,
        DEFAULT_DURATION_TOLERANCE_SECS,
    )
}

/// Matches a list of stream tracks against candidate local tracks with custom duration tolerance.
pub fn match_tracks_against_library_with_tolerance(
    stream_tracks: &[StreamTrack],
    library_tracks: &[Track],
    tolerance_secs: f64,
) -> Vec<StreamTrack> {
    stream_tracks
        .iter()
        .map(|st| {
            let mut updated = st.clone();
            if let Some((matched_track, _)) = match_single_track(st, library_tracks, tolerance_secs)
            {
                updated.set_local_match(matched_track.id, &matched_track.path);
            } else {
                updated.clear_local_match();
            }
            updated
        })
        .collect()
}

/// Executes "Find in my library" against the local SQLite database.
pub fn find_in_my_library(
    db: &Database,
    stream_tracks: &[StreamTrack],
) -> Result<Vec<StreamTrack>, String> {
    let local_tracks = db
        .get_all_tracks()
        .map_err(|e| format!("Failed to query local tracks: {e}"))?;
    Ok(match_tracks_against_library(stream_tracks, &local_tracks))
}

/// Inserts a streaming track into the local `tracks` table with `source = 'stream'` and `url = track.uri`.
/// Returns the resulting track `id`.
pub fn insert_or_get_stream_track(db: &Database, track: &StreamTrack) -> Result<i64, String> {
    let conn = db.lock_conn();
    let path = if !track.uri.trim().is_empty() {
        track.uri.clone()
    } else {
        format!("stream:{}:{}", track.provider, track.id)
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    conn.execute(
        "INSERT INTO tracks (
            path, source, url, thumb, title, artist, albumartist, album,
            duration, added
        ) VALUES (?1, 'stream', ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(path) DO UPDATE SET
            title = excluded.title,
            artist = excluded.artist,
            albumartist = excluded.albumartist,
            album = excluded.album,
            thumb = excluded.thumb,
            duration = excluded.duration",
        rusqlite::params![
            path,
            track.uri,
            track.art_url,
            track.title,
            track.artist,
            track.artist,
            track.album,
            track.duration_secs,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert stream track into library: {e}"))?;

    let mut stmt = conn
        .prepare("SELECT id FROM tracks WHERE path = ?1")
        .map_err(|e| format!("Failed to prepare track id query: {e}"))?;
    let id: i64 = stmt
        .query_row(rusqlite::params![path], |row| row.get(0))
        .map_err(|e| format!("Failed to query stream track id: {e}"))?;

    Ok(id)
}

/// Result of saving an imported stream playlist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistSaveResult {
    pub id: i64,
    pub name: String,
    pub track_count: usize,
}

/// Saves an imported tracklist as a local playlist in SQLite.
/// - For tracks available locally (or matched in library), uses the existing local track ID.
/// - For streaming-only tracks, inserts into `tracks` with `source = 'stream'` and `url = track.uri`.
/// - Adds all items into the playlist via `Database::add_track_to_playlist`.
pub fn save_imported_playlist(
    db: &Database,
    playlist_name: &str,
    tracks: &[StreamTrack],
) -> Result<PlaylistSaveResult, String> {
    let name = if playlist_name.trim().is_empty() {
        "Imported Playlist"
    } else {
        playlist_name.trim()
    };

    let playlist_id = db
        .create_playlist(name)
        .map_err(|e| format!("Failed to create playlist '{name}': {e}"))?;

    let local_tracks = db.get_all_tracks().unwrap_or_default();
    let mut added_count = 0;

    for track in tracks {
        let track_id = if let Some(local_id) = track.local_track_id {
            // Verify track still exists locally
            match db.get_track_by_id(local_id) {
                Ok(Some(_)) => local_id,
                _ => insert_or_get_stream_track(db, track)?,
            }
        } else if let Some((matched, _)) =
            match_single_track(track, &local_tracks, DEFAULT_DURATION_TOLERANCE_SECS)
        {
            // Auto-link to local library track if present
            matched.id
        } else {
            // Streaming-only track
            insert_or_get_stream_track(db, track)?
        };

        db.add_track_to_playlist(playlist_id, track_id)
            .map_err(|e| format!("Failed to add track to playlist: {e}"))?;
        added_count += 1;
    }

    Ok(PlaylistSaveResult {
        id: playlist_id,
        name: name.to_string(),
        track_count: added_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::db::Database;
    use crate::library::model::Track;

    fn make_test_local_track(id: i64, path: &str, title: &str, artist: &str, duration_secs: f64) -> Track {
        Track {
            id,
            path: path.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            album_artist: artist.to_string(),
            album: "Test Album".to_string(),
            genre: "J-Pop".to_string(),
            year: 2021,
            track_number: 1,
            duration_secs,
            bitrate_kbps: 1411,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 25_000_000,
            play_count: 5,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    #[test]
    fn test_cjk_string_matching() {
        // CJK text: Japanese Kanji + Romaji artist
        let s1 = "群青";
        let s2 = "YOASOBI";
        assert_eq!(normalize_string(s1), "群青");
        assert_eq!(normalize_string(s2), "yoasobi");

        // Fullwidth CJK Latin and spacing
        let s_fullwidth = "ＹＯＡＳＯＢＩ　群青";
        assert_eq!(normalize_string(s_fullwidth), "yoasobi 群青");

        // Hiragana & Katakana
        let s_hiragana = "夜に駆ける";
        assert_eq!(normalize_string(s_hiragana), "夜に駆ける");
        let s_katakana = "アイドル";
        assert_eq!(normalize_string(s_katakana), "アイドル");

        // Korean Hangul
        let s_hangul = "봄날 (Spring Day)";
        // " (Spring Day)" is not a noise token, parens removed as punctuation
        assert_eq!(normalize_string(s_hangul), "봄날 spring day");

        // Matching test
        let stream_track = StreamTrack::new(
            "sp_1",
            "群青",
            "YOASOBI",
            "THE BOOK 2",
            262.0,
            None,
            "spotify:track:gunjo",
            "spotify",
        );
        let local_track = make_test_local_track(101, "C:/music/YOASOBI - Gunjo.flac", "群青", "YOASOBI", 263.5);

        let candidates = vec![local_track];
        let matched = match_single_track(&stream_track, &candidates, 5.0);
        assert!(matched.is_some());
        let (trk, score) = matched.unwrap();
        assert_eq!(trk.id, 101);
        assert!(score >= 180.0);
    }

    #[test]
    fn test_noise_token_stripping() {
        // (feat. ...)
        assert_eq!(
            normalize_string("Song Title (feat. Artist B)"),
            "song title"
        );
        // [feat. ...]
        assert_eq!(
            normalize_string("Song Title [feat. Artist B]"),
            "song title"
        );
        // ft. ...
        assert_eq!(
            normalize_string("Song Title ft. Artist B"),
            "song title"
        );
        // (remix)
        assert_eq!(
            normalize_string("Song Title (remix)"),
            "song title"
        );
        // [remastered ...]
        assert_eq!(
            normalize_string("Song Title [remastered 2021]"),
            "song title"
        );
        // (live ...)
        assert_eq!(
            normalize_string("Song Title (live at Wembley)"),
            "song title"
        );
        // (Live)
        assert_eq!(
            normalize_string("Song Title (Live)"),
            "song title"
        );
        // Trailing hyphen noise: " - Remastered"
        assert_eq!(
            normalize_string("Song Title - Remastered"),
            "song title"
        );
        // Punctuation and quotes
        assert_eq!(
            normalize_string("Don't Stop Believin'"),
            "dont stop believin"
        );
    }

    #[test]
    fn test_duration_tolerance_boundary() {
        let stream_track = StreamTrack::new(
            "sp_dur",
            "Tolerance Test",
            "Artist",
            "Album",
            200.0,
            None,
            "spotify:track:dur",
            "spotify",
        );

        // Within boundary: diff = 4.9s (< 5.0s) -> matches
        let c1 = vec![make_test_local_track(1, "path1.flac", "Tolerance Test", "Artist", 204.9)];
        let res1 = match_single_track(&stream_track, &c1, 5.0);
        assert!(res1.is_some());

        // Boundary exact: diff = 5.0s (<= 5.0s) -> matches
        let c2 = vec![make_test_local_track(2, "path2.flac", "Tolerance Test", "Artist", 205.0)];
        let res2 = match_single_track(&stream_track, &c2, 5.0);
        assert!(res2.is_some());

        // Outside boundary: diff = 5.1s (> 5.0s) -> rejects
        let c3 = vec![make_test_local_track(3, "path3.flac", "Tolerance Test", "Artist", 205.1)];
        let res3 = match_single_track(&stream_track, &c3, 5.0);
        assert!(res3.is_none());

        // Outside boundary: diff = 8.0s (> 5.0s) -> rejects
        let c4 = vec![make_test_local_track(4, "path4.flac", "Tolerance Test", "Artist", 208.0)];
        let res4 = match_single_track(&stream_track, &c4, 5.0);
        assert!(res4.is_none());
    }

    #[test]
    fn test_save_as_playlist_integration_in_memory() {
        let db = Database::open_in_memory().expect("open in-memory db");

        // 1. Insert an existing local track into SQLite
        let local_track = make_test_local_track(0, "C:/music/track1.flac", "Local Song", "Local Artist", 180.0);
        let local_id = db.upsert_track(&local_track).expect("upsert local track");

        // 2. Prepare imported stream tracks:
        // Track A is matched to local track
        let mut track_a = StreamTrack::new(
            "sp_a",
            "Local Song",
            "Local Artist",
            "Test Album",
            181.0,
            None,
            "spotify:track:a",
            "spotify",
        );
        track_a.set_local_match(local_id, "C:/music/track1.flac");

        // Track B is a streaming-only track
        let track_b = StreamTrack::new(
            "sp_b",
            "Stream Only Song",
            "Stream Artist",
            "Stream Album",
            210.0,
            Some("https://example.com/art.jpg".to_string()),
            "spotify:track:b",
            "spotify",
        );

        let stream_tracks = vec![track_a, track_b];

        // 3. Save as playlist
        let result = save_imported_playlist(&db, "My Spotify Import", &stream_tracks)
            .expect("save playlist");

        assert_eq!(result.name, "My Spotify Import");
        assert_eq!(result.track_count, 2);
        assert!(result.id > 0);

        // 4. Verify playlist was saved in SQLite
        let playlists = db.get_playlists().expect("get playlists");
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].id, result.id);
        assert_eq!(playlists[0].name, "My Spotify Import");
        assert_eq!(playlists[0].track_count, 2);

        // 5. Verify playlist items
        let playlist_tracks = db.get_playlist_tracks(result.id).expect("get playlist tracks");
        assert_eq!(playlist_tracks.len(), 2);

        // First track is the local track with local path
        assert_eq!(playlist_tracks[0].id, local_id);
        assert_eq!(playlist_tracks[0].path, "C:/music/track1.flac");
        assert_eq!(playlist_tracks[0].title, "Local Song");

        // Second track is the stream track with source = 'stream' and url = track.uri
        assert_eq!(playlist_tracks[1].title, "Stream Only Song");
        assert_eq!(playlist_tracks[1].artist, "Stream Artist");
        assert_eq!(playlist_tracks[1].path, "spotify:track:b");
    }

    #[test]
    fn test_match_tracks_against_library_bulk() {
        let local_1 = make_test_local_track(1, "path1.flac", "Levitating", "Dua Lipa", 203.0);
        let local_2 = make_test_local_track(2, "path2.flac", "Blinding Lights", "The Weeknd", 200.0);
        let local_tracks = vec![local_1, local_2];

        let stream_1 = StreamTrack::new(
            "sp_1",
            "Levitating (feat. DaBaby)",
            "Dua Lipa",
            "Future Nostalgia",
            203.5,
            None,
            "spotify:track:1",
            "spotify",
        );
        let stream_2 = StreamTrack::new(
            "sp_2",
            "Unmatched Song",
            "Unknown Artist",
            "Unknown Album",
            150.0,
            None,
            "spotify:track:2",
            "spotify",
        );

        let matched = match_tracks_against_library(&[stream_1, stream_2], &local_tracks);
        assert_eq!(matched.len(), 2);
        assert!(matched[0].is_available_locally());
        assert_eq!(matched[0].local_track_id, Some(1));
        assert_eq!(matched[0].local_path.as_deref(), Some("path1.flac"));

        assert!(!matched[1].is_available_locally());
        assert_eq!(matched[1].local_track_id, None);
        assert_eq!(matched[1].local_path, None);
    }
}
