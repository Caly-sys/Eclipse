use crate::lyrics::lrc::{LyricLine, Lyrics};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const LRCLIB_BASE_URL: &str = "https://lrclib.net";
pub const USER_AGENT: &str = "Umbra/0.1.0 (https://github.com/sadult/umbra)";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LrclibRecord {
    pub id: u64,
    pub name: Option<String>,
    pub track_name: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    #[serde(default)]
    pub instrumental: bool,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
}

impl LrclibRecord {
    pub fn to_lyrics(&self) -> Lyrics {
        if self.instrumental {
            return Lyrics {
                lines: vec![LyricLine {
                    timestamp_ms: 0,
                    text: "Instrumental".to_string(),
                }],
                is_synced: false,
                source: "LRCLIB (Instrumental)".to_string(),
            };
        }

        if let Some(ref synced) = self.synced_lyrics {
            if !synced.trim().is_empty() {
                return Lyrics::parse(synced, "LRCLIB (Synced)");
            }
        }

        if let Some(ref plain) = self.plain_lyrics {
            if !plain.trim().is_empty() {
                return Lyrics::parse(plain, "LRCLIB (Plain)");
            }
        }

        Lyrics {
            lines: Vec::new(),
            is_synced: false,
            source: "LRCLIB (Empty)".to_string(),
        }
    }
}

pub fn lyrics_cache_dir() -> PathBuf {
    let dir = crate::config::app_data_dir().join("lyrics_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn lyrics_cache_path(track_path: &Path) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    track_path.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();
    lyrics_cache_dir().join(format!("{:016x}.lrc", hash))
}

pub fn save_to_cache(track_path: &Path, content: &str) {
    if content.trim().is_empty() {
        return;
    }
    let path = lyrics_cache_path(track_path);
    let _ = std::fs::write(&path, content);
}

pub fn load_from_cache(track_path: &Path) -> Option<String> {
    // 1. Try hashed path
    let path = lyrics_cache_path(track_path);
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if !content.trim().is_empty() {
                return Some(content);
            }
        }
    }

    // 2. Try clean track file stem name in cache directory
    if let Some(stem) = track_path.file_stem().and_then(|s| s.to_str()) {
        let clean: String = stem
            .chars()
            .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let stem_path = lyrics_cache_dir().join(format!("{}.lrc", clean.trim()));
        if stem_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&stem_path) {
                if !content.trim().is_empty() {
                    return Some(content);
                }
            }
        }
    }

    None
}

pub fn apply_and_cache_record(track_path: &Path, record: &LrclibRecord) -> Lyrics {
    if let Some(ref synced) = record.synced_lyrics {
        save_to_cache(track_path, synced);
    } else if let Some(ref plain) = record.plain_lyrics {
        save_to_cache(track_path, plain);
    } else if record.instrumental {
        save_to_cache(track_path, "[00:00.00] Instrumental\n");
    }
    record.to_lyrics()
}

// Step B: Exact GET
pub fn fetch_exact(
    artist: &str,
    title: &str,
    album: &str,
    duration_secs: f64,
) -> Option<LrclibRecord> {
    let clean_artist = artist.trim();
    let clean_title = title.trim();
    if clean_title.is_empty() {
        return None;
    }

    let mut url = format!(
        "{}/api/get?artist_name={}&track_name={}",
        LRCLIB_BASE_URL,
        urlencoding(clean_artist),
        urlencoding(clean_title)
    );
    if !album.trim().is_empty() {
        url.push_str(&format!("&album_name={}", urlencoding(album.trim())));
    }
    if duration_secs > 0.0 {
        url.push_str(&format!("&duration={}", duration_secs.round() as u64));
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .user_agent(USER_AGENT)
        .build();

    let res = agent.get(&url).call().ok()?;
    let record: LrclibRecord = res.into_json().ok()?;
    Some(record)
}

// Step C: Fallback smart search
pub fn fetch_smart_search(
    artist: &str,
    title: &str,
    duration_secs: f64,
) -> Option<LrclibRecord> {
    let clean_artist = artist.trim();
    let clean_title = title.trim();
    if clean_title.is_empty() {
        return None;
    }

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(6))
        .user_agent(USER_AGENT)
        .build();

    let url1 = format!(
        "{}/api/search?track_name={}&artist_name={}",
        LRCLIB_BASE_URL,
        urlencoding(clean_title),
        urlencoding(clean_artist)
    );

    let mut records: Vec<LrclibRecord> = agent
        .get(&url1)
        .call()
        .ok()
        .and_then(|r| r.into_json().ok())
        .unwrap_or_default();

    if records.is_empty() {
        let q = format!("{} {}", clean_title, clean_artist);
        let url2 = format!("{}/api/search?q={}", LRCLIB_BASE_URL, urlencoding(q.trim()));
        if let Ok(res) = agent.get(&url2).call() {
            if let Ok(list) = res.into_json::<Vec<LrclibRecord>>() {
                records = list;
            }
        }
    }

    if records.is_empty() {
        return None;
    }

    pick_best_record(records, duration_secs)
}

pub fn pick_best_record(records: Vec<LrclibRecord>, target_duration: f64) -> Option<LrclibRecord> {
    if records.is_empty() {
        return None;
    }

    // 1. Records with non-empty synced_lyrics and duration within +-5.0s
    let mut synced_matches: Vec<(f64, LrclibRecord)> = records
        .iter()
        .filter(|r| {
            if let Some(ref s) = r.synced_lyrics {
                if !s.trim().is_empty() {
                    if target_duration > 0.0 {
                        if let Some(dur) = r.duration {
                            return (dur - target_duration).abs() <= 5.0;
                        }
                    }
                    return true;
                }
            }
            false
        })
        .map(|r| {
            let diff = r.duration.map(|d| (d - target_duration).abs()).unwrap_or(999.0);
            (diff, r.clone())
        })
        .collect();

    if !synced_matches.is_empty() {
        synced_matches.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        return Some(synced_matches.remove(0).1);
    }

    // 2. Any record with synced_lyrics
    if let Some(rec) = records.iter().find(|r| {
        r.synced_lyrics.as_ref().is_some_and(|s| !s.trim().is_empty())
    }) {
        return Some(rec.clone());
    }

    // 3. Fallback to plain_lyrics (preferably closest duration)
    let mut plain_matches: Vec<(f64, LrclibRecord)> = records
        .iter()
        .filter(|r| {
            r.plain_lyrics.as_ref().is_some_and(|p| !p.trim().is_empty())
        })
        .map(|r| {
            let diff = r.duration.map(|d| (d - target_duration).abs()).unwrap_or(999.0);
            (diff, r.clone())
        })
        .collect();

    if !plain_matches.is_empty() {
        plain_matches.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        return Some(plain_matches.remove(0).1);
    }

    // 4. Fallback to instrumental
    if let Some(rec) = records.iter().find(|r| r.instrumental) {
        return Some(rec.clone());
    }

    None
}

pub fn fetch_online_record(
    artist: &str,
    title: &str,
    album: &str,
    duration_secs: f64,
) -> Option<LrclibRecord> {
    // Step B: Exact GET
    if let Some(exact) = fetch_exact(artist, title, album, duration_secs) {
        return Some(exact);
    }

    // Step C: Fallback search
    fetch_smart_search(artist, title, duration_secs)
}

pub fn fetch_online_with_fallback(
    artist: &str,
    title: &str,
    album: &str,
    duration_secs: f64,
) -> Option<Lyrics> {
    fetch_online_record(artist, title, album, duration_secs).map(|r| r.to_lyrics())
}

pub fn fetch_online(
    artist: &str,
    title: &str,
    album: &str,
    duration_secs: f64,
) -> Option<Lyrics> {
    fetch_online_with_fallback(artist, title, album, duration_secs)
}

pub fn fetch_online_with_fallback_and_cache(
    track_path: &Path,
    artist: &str,
    title: &str,
    album: &str,
    duration_secs: f64,
) -> Option<Lyrics> {
    let record = fetch_online_record(artist, title, album, duration_secs)?;
    Some(apply_and_cache_record(track_path, &record))
}

pub fn fetch_track_lyrics_async<F>(
    track_path: PathBuf,
    artist: String,
    title: String,
    album: String,
    duration_secs: f64,
    callback: F,
) -> std::thread::JoinHandle<()>
where
    F: FnOnce(Option<Lyrics>) + Send + 'static,
{
    std::thread::spawn(move || {
        // Step A: Local first
        if let Some(local) = crate::lyrics::lrc::Lyrics::load_with_cache(&track_path) {
            callback(Some(local));
            return;
        }

        // Steps B & C: Online fetch with fallback & cache
        let res = fetch_online_with_fallback_and_cache(
            &track_path,
            &artist,
            &title,
            &album,
            duration_secs,
        );
        callback(res);
    })
}

pub fn search_lrclib_manual(
    query: &str,
    track_name: Option<&str>,
    artist_name: Option<&str>,
) -> Result<Vec<LrclibRecord>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .user_agent(USER_AGENT)
        .build();

    let clean_q = query.trim();
    let clean_t = track_name.map(|s| s.trim()).filter(|s| !s.is_empty());
    let clean_a = artist_name.map(|s| s.trim()).filter(|s| !s.is_empty());

    let url = if let (Some(t), Some(a)) = (clean_t, clean_a) {
        format!(
            "{}/api/search?track_name={}&artist_name={}",
            LRCLIB_BASE_URL,
            urlencoding(t),
            urlencoding(a)
        )
    } else if let Some(t) = clean_t {
        format!("{}/api/search?track_name={}", LRCLIB_BASE_URL, urlencoding(t))
    } else if let Some(a) = clean_a {
        format!("{}/api/search?artist_name={}", LRCLIB_BASE_URL, urlencoding(a))
    } else if !clean_q.is_empty() {
        format!("{}/api/search?q={}", LRCLIB_BASE_URL, urlencoding(clean_q))
    } else {
        return Ok(Vec::new());
    };

    let res = agent
        .get(&url)
        .call()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let records: Vec<LrclibRecord> = res
        .into_json()
        .map_err(|e| format!("JSON parsing failed: {}", e))?;

    if records.is_empty() && !clean_q.is_empty() && (clean_t.is_some() || clean_a.is_some()) {
        let retry_url = format!("{}/api/search?q={}", LRCLIB_BASE_URL, urlencoding(clean_q));
        if let Ok(res2) = agent.get(&retry_url).call() {
            if let Ok(records2) = res2.into_json::<Vec<LrclibRecord>>() {
                return Ok(records2);
            }
        }
    }

    Ok(records)
}

pub fn urlencoding(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_to_lyrics_synced() {
        let record = LrclibRecord {
            id: 100,
            name: Some("Test Track".to_string()),
            track_name: "Test Track".to_string(),
            artist_name: "Test Artist".to_string(),
            album_name: Some("Test Album".to_string()),
            duration: Some(180.0),
            instrumental: false,
            plain_lyrics: Some("Plain lyrics".to_string()),
            synced_lyrics: Some("[00:10.00] Synced line\n[00:20.00] Next line".to_string()),
        };
        let lyrics = record.to_lyrics();
        assert!(lyrics.is_synced);
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].text, "Synced line");
        assert_eq!(lyrics.lines[0].timestamp_ms, 10000);
    }

    #[test]
    fn test_record_to_lyrics_plain() {
        let record = LrclibRecord {
            id: 101,
            name: None,
            track_name: "Test Track".to_string(),
            artist_name: "Test Artist".to_string(),
            album_name: None,
            duration: Some(120.0),
            instrumental: false,
            plain_lyrics: Some("Line 1\nLine 2".to_string()),
            synced_lyrics: None,
        };
        let lyrics = record.to_lyrics();
        assert!(!lyrics.is_synced);
        assert_eq!(lyrics.lines.len(), 2);
        assert_eq!(lyrics.lines[0].text, "Line 1");
    }

    #[test]
    fn test_record_to_lyrics_instrumental() {
        let record = LrclibRecord {
            id: 102,
            name: None,
            track_name: "Instrumental Piece".to_string(),
            artist_name: "Composer".to_string(),
            album_name: None,
            duration: Some(200.0),
            instrumental: true,
            plain_lyrics: None,
            synced_lyrics: None,
        };
        let lyrics = record.to_lyrics();
        assert_eq!(lyrics.lines.len(), 1);
        assert_eq!(lyrics.lines[0].text, "Instrumental");
    }

    #[test]
    fn test_pick_best_record_duration_filter() {
        let rec_close = LrclibRecord {
            id: 1,
            name: None,
            track_name: "Song".to_string(),
            artist_name: "Artist".to_string(),
            album_name: None,
            duration: Some(182.0),
            instrumental: false,
            plain_lyrics: None,
            synced_lyrics: Some("[00:05.00] Hi".to_string()),
        };
        let rec_far = LrclibRecord {
            id: 2,
            name: None,
            track_name: "Song".to_string(),
            artist_name: "Artist".to_string(),
            album_name: None,
            duration: Some(250.0),
            instrumental: false,
            plain_lyrics: None,
            synced_lyrics: Some("[00:05.00] Hi far".to_string()),
        };

        let target_duration = 180.0;
        let chosen = pick_best_record(vec![rec_far, rec_close.clone()], target_duration);
        assert!(chosen.is_some());
        assert_eq!(chosen.unwrap().id, 1);
    }

    #[test]
    fn test_pick_best_record_fallback_plain() {
        let rec_plain = LrclibRecord {
            id: 10,
            name: None,
            track_name: "Song".to_string(),
            artist_name: "Artist".to_string(),
            album_name: None,
            duration: Some(180.0),
            instrumental: false,
            plain_lyrics: Some("Just words".to_string()),
            synced_lyrics: None,
        };
        let chosen = pick_best_record(vec![rec_plain.clone()], 180.0);
        assert!(chosen.is_some());
        assert_eq!(chosen.unwrap().id, 10);
    }

    #[test]
    fn test_disk_cache_save_and_load() {
        let dummy_path = Path::new("C:/test_music/fake_artist/test_song.flac");
        let lrc_content = "[00:10.00] Cached lyric line\n[00:20.00] Another line";

        save_to_cache(dummy_path, lrc_content);
        let loaded = load_from_cache(dummy_path);
        assert_eq!(loaded.as_deref(), Some(lrc_content));

        // Clean up
        let cache_file = lyrics_cache_path(dummy_path);
        let _ = std::fs::remove_file(cache_file);
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("Aimyon"), "Aimyon");
        assert_eq!(urlencoding("Track & Title"), "Track%20%26%20Title");
        assert_eq!(urlencoding("Hello World"), "Hello%20World");
    }
}
