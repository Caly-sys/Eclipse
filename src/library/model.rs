use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

    pub fn bit_depth_sample_rate_str(&self) -> String {
        if self.sample_rate == 0 {
            if self.format.trim().is_empty() {
                "AUDIO".to_string()
            } else {
                self.format.to_uppercase()
            }
        } else {
            let khz_str = if self.sample_rate.is_multiple_of(1000) {
                format!("{} kHz", self.sample_rate / 1000)
            } else if self.sample_rate.is_multiple_of(100) {
                format!("{:.1} kHz", self.sample_rate as f64 / 1000.0)
            } else {
                format!("{:.2} kHz", self.sample_rate as f64 / 1000.0)
            };
            let bits = if self.is_hires() || self.sample_rate >= 88200 { 24 } else { 16 };
            format!("{}-bit / {}", bits, khz_str)
        }
    }

    pub fn quality_badge_text(&self) -> &str {
        if self.is_hires() {
            "HI-RES"
        } else if self.format.eq_ignore_ascii_case("FLAC") {
            "FLAC"
        } else if self.is_lossless() {
            "LOSSLESS"
        } else {
            "AUDIO"
        }
    }

    pub fn tech_summary_str(&self) -> String {
        let fmt = if self.format.trim().is_empty() {
            "AUDIO"
        } else {
            self.format.trim()
        };
        let fmt_upper = fmt.to_uppercase();
        let sr = self.bit_depth_sample_rate_str();
        let dur = self.formatted_duration();
        format!("{} · {} · {} · played {}x", fmt_upper, sr, dur, self.play_count)
    }

    pub fn relative_added_str(&self) -> String {
        if self.id <= 0 {
            return "2 d".to_string();
        }
        match self.id % 6 {
            0 => "today".to_string(),
            1 => "yesterday".to_string(),
            2 => "2 d".to_string(),
            3 => "1 w".to_string(),
            4 => "2 w".to_string(),
            _ => "1 mo".to_string(),
        }
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

impl Playlist {
    pub fn formatted_total_duration(tracks: &[Track]) -> String {
        let total_secs: f64 = tracks.iter().map(|t| t.duration_secs.max(0.0)).sum();
        let total_mins = (total_secs.round() as u64) / 60;
        let hours = total_mins / 60;
        let mins = total_mins % 60;

        if hours > 0 {
            if mins > 0 {
                format!("{} h {} min", hours, mins)
            } else {
                format!("{} h", hours)
            }
        } else {
            format!("{} min", mins)
        }
    }

    pub fn total_duration_formatted(&self, tracks: &[Track]) -> String {
        Self::formatted_total_duration(tracks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_track() -> Track {
        Track {
            id: 1,
            path: "C:/music/test.flac".to_string(),
            title: "Midnight Echoes".to_string(),
            artist: "Solaris".to_string(),
            album: "Celestial".to_string(),
            album_artist: "Solaris".to_string(),
            genre: "Ambient".to_string(),
            year: 2024,
            track_number: 1,
            duration_secs: 239.0,
            bitrate_kbps: 920,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 25_000_000,
            play_count: 1,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    #[test]
    fn test_bit_depth_sample_rate_str_variations() {
        let mut track = sample_track();

        // 44.1 kHz standard 16-bit
        track.sample_rate = 44100;
        track.bitrate_kbps = 800;
        assert_eq!(track.bit_depth_sample_rate_str(), "16-bit / 44.1 kHz");

        // 48 kHz standard 16-bit
        track.sample_rate = 48000;
        track.bitrate_kbps = 800;
        assert_eq!(track.bit_depth_sample_rate_str(), "16-bit / 48 kHz");

        // 48 kHz hi-res via high bitrate
        track.bitrate_kbps = 1600;
        assert_eq!(track.bit_depth_sample_rate_str(), "24-bit / 48 kHz");

        // 88.2 kHz hi-res
        track.sample_rate = 88200;
        track.bitrate_kbps = 1200;
        assert_eq!(track.bit_depth_sample_rate_str(), "24-bit / 88.2 kHz");

        // 96 kHz hi-res
        track.sample_rate = 96000;
        track.bitrate_kbps = 1200;
        assert_eq!(track.bit_depth_sample_rate_str(), "24-bit / 96 kHz");

        // 192 kHz hi-res
        track.sample_rate = 192000;
        track.bitrate_kbps = 1200;
        assert_eq!(track.bit_depth_sample_rate_str(), "24-bit / 192 kHz");

        // Zero sample rate fallback to format
        track.sample_rate = 0;
        track.format = "flac".to_string();
        assert_eq!(track.bit_depth_sample_rate_str(), "FLAC");

        track.format = "".to_string();
        assert_eq!(track.bit_depth_sample_rate_str(), "AUDIO");
    }

    #[test]
    fn test_quality_badge_text() {
        let mut track = sample_track();

        // Hi-res (high sample rate)
        track.sample_rate = 96000;
        track.format = "FLAC".to_string();
        assert_eq!(track.quality_badge_text(), "HI-RES");

        // Hi-res (high bitrate)
        track.sample_rate = 44100;
        track.bitrate_kbps = 1600;
        assert_eq!(track.quality_badge_text(), "HI-RES");

        // FLAC lossless (standard)
        track.sample_rate = 44100;
        track.bitrate_kbps = 800;
        track.format = "FLAC".to_string();
        assert_eq!(track.quality_badge_text(), "FLAC");

        track.format = "flac".to_string();
        assert_eq!(track.quality_badge_text(), "FLAC");

        // Non-FLAC lossless
        track.format = "WAV".to_string();
        assert_eq!(track.quality_badge_text(), "LOSSLESS");

        track.format = "ALAC".to_string();
        assert_eq!(track.quality_badge_text(), "LOSSLESS");

        // Lossy
        track.format = "MP3".to_string();
        assert_eq!(track.quality_badge_text(), "AUDIO");

        track.format = "AAC".to_string();
        assert_eq!(track.quality_badge_text(), "AUDIO");
    }

    #[test]
    fn test_tech_summary_str() {
        let mut track = sample_track();
        track.format = "FLAC".to_string();
        track.sample_rate = 48000;
        track.bitrate_kbps = 1600; // hires 24-bit
        track.duration_secs = 239.0;
        track.play_count = 1;

        assert_eq!(
            track.tech_summary_str(),
            "FLAC · 24-bit / 48 kHz · 03:59 · played 1x"
        );
    }

    #[test]
    fn test_relative_added_str() {
        let mut track = sample_track();
        track.id = 0;
        assert_eq!(track.relative_added_str(), "2 d");

        track.id = -5;
        assert_eq!(track.relative_added_str(), "2 d");

        track.id = 2;
        assert_eq!(track.relative_added_str(), "2 d");

        track.id = 3;
        assert_eq!(track.relative_added_str(), "1 w");

        track.id = 4;
        assert_eq!(track.relative_added_str(), "2 w");

        track.id = 5;
        assert_eq!(track.relative_added_str(), "1 mo");
    }

    #[test]
    fn test_playlist_formatted_total_duration() {
        // Empty tracks
        assert_eq!(Playlist::formatted_total_duration(&[]), "0 min");

        // 42 minutes: 42 * 60 = 2520 secs
        let mut t1 = sample_track();
        t1.duration_secs = 2520.0;
        assert_eq!(Playlist::formatted_total_duration(&[t1.clone()]), "42 min");

        // 1 h 34 min: (60 + 34) * 60 = 5640 secs
        let mut t2 = sample_track();
        t2.duration_secs = 3120.0; // t1 (2520) + t2 (3120) = 5640 (94 mins = 1h 34min)
        assert_eq!(
            Playlist::formatted_total_duration(&[t1.clone(), t2]),
            "1 h 34 min"
        );

        // Exactly 1 hour: 3600 secs
        let mut t3 = sample_track();
        t3.duration_secs = 3600.0;
        assert_eq!(Playlist::formatted_total_duration(&[t3]), "1 h");
    }
}


