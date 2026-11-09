use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub genre: String,
    pub year: u32,
    pub track_number: u32,
    pub duration_secs: f64,
    pub bitrate_kbps: u32,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
    pub file_size: u64,
    pub play_count: u32,
    pub favorite: bool,
    pub has_art: bool,
    pub folder: String,
}

impl Track {
    pub fn display_title(&self) -> &str {
        if self.title.trim().is_empty() {
            if let Some(name) = std::path::Path::new(&self.path).file_stem() {
                return name.to_str().unwrap_or("Unknown Title");
            }
            "Unknown Title"
        } else {
            &self.title
        }
    }

    pub fn display_artist(&self) -> &str {
        if !self.artist.trim().is_empty() {
            &self.artist
        } else if !self.album_artist.trim().is_empty() {
            &self.album_artist
        } else {
            "Unknown Artist"
        }
    }

    pub fn display_album(&self) -> &str {
        if self.album.trim().is_empty() {
            "Unknown Album"
        } else {
            &self.album
        }
    }

    pub fn formatted_duration(&self) -> String {
        format_time_secs(self.duration_secs)
    }

    pub fn is_lossless(&self) -> bool {
        let f = self.format.to_uppercase();
        f == "FLAC" || f == "WAV" || f == "ALAC" || f == "AIFF" || f == "WV"
    }

    pub fn is_hires(&self) -> bool {
        self.sample_rate >= 88200 || self.bitrate_kbps > 1500
    }
}

pub fn format_time_secs(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let mins = total / 60;
    let remaining = total % 60;
    format!("{:02}:{:02}", mins, remaining)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created: f64,
    pub description: String,
    pub track_count: usize,
}

