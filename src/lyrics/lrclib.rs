use crate::lyrics::lrc::Lyrics;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct LrclibResponse {
    pub id: Option<i64>,
    #[serde(rename = "trackName")]
    pub track_name: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    pub instrumental: Option<bool>,
    #[serde(rename = "plainLyrics")]
    pub plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    pub synced_lyrics: Option<String>,
}

pub fn fetch_online(artist: &str, title: &str, album: &str, duration_secs: f64) -> Option<Lyrics> {
    let clean_artist = artist.trim();
    let clean_title = title.trim();
    if clean_title.is_empty() {
        return None;
    }

    let url = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}&album_name={}&duration={}",
        urlencoding(clean_artist),
        urlencoding(clean_title),
        urlencoding(album.trim()),
        duration_secs.round() as u64
    );

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(5))
        .user_agent("Umbra/1.2.0 (Rust; +https://github.com/sadult/umbra)")
        .build();

    let res = agent.get(&url).call().ok()?;
    let data: LrclibResponse = res.into_json().ok()?;

    if let Some(true) = data.instrumental {
        return Some(Lyrics {
            lines: vec![crate::lyrics::lrc::LyricLine {
                timestamp_ms: 0,
                text: "Instrumental".to_string(),
            }],
            is_synced: false,
            source: "LRCLIB (Instrumental)".to_string(),
        });
    }

    if let Some(synced) = data.synced_lyrics {
        if !synced.trim().is_empty() {
            return Some(Lyrics::parse(&synced, "LRCLIB (Synced)"));
        }
    }

    if let Some(plain) = data.plain_lyrics {
        if !plain.trim().is_empty() {
            return Some(Lyrics::parse(&plain, "LRCLIB (Plain)"));
        }
    }

    None
}

fn urlencoding(s: &str) -> String {
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

