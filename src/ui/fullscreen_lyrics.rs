//! Fullscreen synchronized lyrics data structures and precise timestamp lookup.
//!
//! Provides `FullscreenLyrics` and `FullscreenLyricLine` for calculating
//! the active lyric line index based on playback position in seconds,
//! supporting a 50ms early-highlight threshold, instrumental breaks,
//! and strict ascending timestamp ordering.

#[derive(Debug, Clone, PartialEq)]
pub struct FullscreenLyricLine {
    pub time_secs: f64,
    pub text: String,
    pub is_instrumental: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FullscreenLyrics {
    pub lines: Vec<FullscreenLyricLine>,
}

/// Type alias for compatibility with parsed lyrics.
pub type ParsedLyrics = crate::lyrics::Lyrics;

impl FullscreenLyrics {
    /// Creates a new `FullscreenLyrics` instance, ensuring lines are strictly sorted
    /// ascending by `time_secs`.
    pub fn new(mut lines: Vec<FullscreenLyricLine>) -> Self {
        lines.sort_by(|a, b| a.time_secs.partial_cmp(&b.time_secs).unwrap_or(std::cmp::Ordering::Equal));
        Self { lines }
    }

    /// Construct from project's `crate::lyrics::Lyrics` (or `crate::lyrics::lrc::Lyrics`).
    pub fn from_lyrics(lrc: &crate::lyrics::Lyrics) -> Self {
        let mut lines = Vec::new();
        for line in &lrc.lines {
            let time_secs = (line.timestamp_ms as f64) / 1000.0;
            let text = line.text.trim().to_string();
            let is_instrumental = text == "..." || text.is_empty();
            lines.push(FullscreenLyricLine {
                time_secs,
                text,
                is_instrumental,
            });
        }
        Self::new(lines)
    }

    /// Alias for `from_lyrics` matching shared contract.
    pub fn from_parsed_lrc(lrc: &crate::lyrics::Lyrics) -> Self {
        Self::from_lyrics(lrc)
    }

    /// Construct by parsing standard LRC string content directly.
    pub fn from_lrc_str(content: &str) -> Self {
        let parsed = crate::lyrics::lrc::Lyrics::parse(content, "fullscreen_lyrics");
        Self::from_lyrics(&parsed)
    }

    /// Returns the active lyric line index for the given playback position in seconds.
    ///
    /// Rules:
    /// - Returns the last line where `line.time_secs <= current_pos_secs + 0.05`.
    /// - Returns `None` if `current_pos_secs + 0.05 < first_line.time_secs`.
    /// - Returns `Some(last_idx)` if `current_pos_secs` is past the final line.
    /// - Returns `None` if `lines` is empty or `current_pos_secs` is NaN.
    /// - Never panics; handles empty or out-of-order lines safely.
    pub fn active_line_index(&self, current_pos_secs: f64) -> Option<usize> {
        if self.lines.is_empty() || current_pos_secs.is_nan() {
            return None;
        }

        let threshold = current_pos_secs + 0.05;
        let first_line_time = self.lines.first()?.time_secs;

        if threshold < first_line_time {
            return None;
        }

        // Using partition_point to find the first line strictly > threshold,
        // then the active line is the one immediately preceding it.
        let idx = self.lines.partition_point(|line| line.time_secs <= threshold);
        if idx == 0 {
            None
        } else {
            Some(idx - 1)
        }
    }

    /// Returns a reference to the active lyric line if one exists.
    pub fn active_line(&self, current_pos_secs: f64) -> Option<&FullscreenLyricLine> {
        self.active_line_index(current_pos_secs).and_then(|idx| self.lines.get(idx))
    }

    /// Returns true if there are no lyric lines.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the number of lyric lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns a reference to the lyric line at the specified index.
    pub fn get(&self, index: usize) -> Option<&FullscreenLyricLine> {
        self.lines.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lyrics() -> FullscreenLyrics {
        FullscreenLyrics::new(vec![
            FullscreenLyricLine {
                time_secs: 10.0,
                text: "First line of the song".to_string(),
                is_instrumental: false,
            },
            FullscreenLyricLine {
                time_secs: 15.0,
                text: "...".to_string(),
                is_instrumental: true,
            },
            FullscreenLyricLine {
                time_secs: 20.0,
                text: "Second line of the song".to_string(),
                is_instrumental: false,
            },
            FullscreenLyricLine {
                time_secs: 30.0,
                text: "Third line of the song".to_string(),
                is_instrumental: false,
            },
        ])
    }

    #[test]
    fn test_timestamp_before_first_line_returns_none() {
        let lyrics = sample_lyrics();
        // 0.0s is well before 10.0s (first line)
        assert_eq!(lyrics.active_line_index(0.0), None);
        // 5.0s is before 10.0s
        assert_eq!(lyrics.active_line_index(5.0), None);
        // 9.94s is before threshold (9.94 + 0.05 = 9.99 < 10.0)
        assert_eq!(lyrics.active_line_index(9.94), None);
    }

    #[test]
    fn test_exact_match_at_line_0_returns_some_0() {
        let lyrics = sample_lyrics();
        // Exact match at line 0 (10.0s)
        assert_eq!(lyrics.active_line_index(10.0), Some(0));
        assert_eq!(
            lyrics.active_line(10.0).map(|l| l.text.as_str()),
            Some("First line of the song")
        );
    }

    #[test]
    fn test_midpoint_timestamp_between_lines_returns_earlier() {
        let lyrics = sample_lyrics();
        // 12.5s is between 10.0s and 15.0s -> returns line 0
        assert_eq!(lyrics.active_line_index(12.5), Some(0));
        // 17.5s is between 15.0s and 20.0s -> returns line 1 (the instrumental line)
        assert_eq!(lyrics.active_line_index(17.5), Some(1));
        // 25.0s is between 20.0s and 30.0s -> returns line 2
        assert_eq!(lyrics.active_line_index(25.0), Some(2));
    }

    #[test]
    fn test_instrumental_placeholder_preserves_ordering() {
        let lyrics = sample_lyrics();
        // Exactly at instrumental line (15.0s)
        assert_eq!(lyrics.active_line_index(15.0), Some(1));
        let line = lyrics.active_line(15.0);
        assert_eq!(line.map(|l| l.is_instrumental), Some(true));
        assert_eq!(line.map(|l| l.text.as_str()), Some("..."));

        // Just after instrumental line (15.5s)
        assert_eq!(lyrics.active_line_index(15.5), Some(1));

        // When line 2 starts at 20.0s
        assert_eq!(lyrics.active_line_index(20.0), Some(2));
        let next_line = lyrics.active_line(20.0);
        assert_eq!(next_line.map(|l| l.is_instrumental), Some(false));
        assert_eq!(next_line.map(|l| l.text.as_str()), Some("Second line of the song"));
    }

    #[test]
    fn test_timestamp_past_final_line_returns_last_idx() {
        let lyrics = sample_lyrics();
        // Exact final line timestamp (30.0s)
        assert_eq!(lyrics.active_line_index(30.0), Some(3));
        // Well past final line (35.0s, 100.0s)
        assert_eq!(lyrics.active_line_index(35.0), Some(3));
        assert_eq!(lyrics.active_line_index(100.0), Some(3));
        assert_eq!(
            lyrics.active_line(100.0).map(|l| l.text.as_str()),
            Some("Third line of the song")
        );
    }

    #[test]
    fn test_empty_lyrics_returns_none() {
        let empty = FullscreenLyrics::default();
        assert_eq!(empty.active_line_index(0.0), None);
        assert_eq!(empty.active_line_index(10.0), None);
        assert_eq!(empty.active_line_index(100.0), None);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_single_line_lyrics() {
        let single = FullscreenLyrics::new(vec![FullscreenLyricLine {
            time_secs: 5.0,
            text: "Only line".to_string(),
            is_instrumental: false,
        }]);

        assert_eq!(single.active_line_index(0.0), None);
        assert_eq!(single.active_line_index(4.94), None);
        assert_eq!(single.active_line_index(4.95), Some(0)); // 50ms early threshold
        assert_eq!(single.active_line_index(5.0), Some(0));
        assert_eq!(single.active_line_index(10.0), Some(0));
    }

    #[test]
    fn test_early_highlight_threshold_boundary() {
        let lyrics = FullscreenLyrics::new(vec![
            FullscreenLyricLine {
                time_secs: 10.0,
                text: "Line A".to_string(),
                is_instrumental: false,
            },
            FullscreenLyricLine {
                time_secs: 20.0,
                text: "Line B".to_string(),
                is_instrumental: false,
            },
        ]);

        // At 9.94s: threshold 9.99 < 10.0 -> None
        assert_eq!(lyrics.active_line_index(9.94), None);
        // At 9.95s: threshold 10.00 == 10.0 -> Some(0)
        assert_eq!(lyrics.active_line_index(9.95), Some(0));

        // At 19.94s: threshold 19.99 < 20.0 -> Still Line A (Some(0))
        assert_eq!(lyrics.active_line_index(19.94), Some(0));
        // At 19.95s: threshold 20.00 == 20.0 -> Switches to Line B (Some(1))
        assert_eq!(lyrics.active_line_index(19.95), Some(1));
    }

    #[test]
    fn test_nan_and_negative_timestamps() {
        let lyrics = sample_lyrics();
        assert_eq!(lyrics.active_line_index(f64::NAN), None);
        assert_eq!(lyrics.active_line_index(-5.0), None);
    }

    #[test]
    fn test_unsorted_lines_sorted_on_new() {
        let unsorted = FullscreenLyrics::new(vec![
            FullscreenLyricLine {
                time_secs: 30.0,
                text: "Line 3".to_string(),
                is_instrumental: false,
            },
            FullscreenLyricLine {
                time_secs: 10.0,
                text: "Line 1".to_string(),
                is_instrumental: false,
            },
            FullscreenLyricLine {
                time_secs: 20.0,
                text: "Line 2".to_string(),
                is_instrumental: false,
            },
        ]);

        assert_eq!(unsorted.lines[0].time_secs, 10.0);
        assert_eq!(unsorted.lines[1].time_secs, 20.0);
        assert_eq!(unsorted.lines[2].time_secs, 30.0);
        assert_eq!(unsorted.active_line_index(15.0), Some(0));
        assert_eq!(unsorted.active_line_index(25.0), Some(1));
        assert_eq!(unsorted.active_line_index(35.0), Some(2));
    }

    #[test]
    fn test_from_lyrics_conversion() {
        let raw = crate::lyrics::Lyrics {
            is_synced: true,
            lines: vec![
                crate::lyrics::LyricLine {
                    timestamp_ms: 1000,
                    text: "Start".to_string(),
                },
                crate::lyrics::LyricLine {
                    timestamp_ms: 2500,
                    text: "...".to_string(),
                },
                crate::lyrics::LyricLine {
                    timestamp_ms: 4000,
                    text: "".to_string(),
                },
                crate::lyrics::LyricLine {
                    timestamp_ms: 5000,
                    text: "End".to_string(),
                },
            ],
            ..Default::default()
        };

        let fs = FullscreenLyrics::from_lyrics(&raw);
        assert_eq!(fs.len(), 4);
        assert_eq!(fs.lines[0].time_secs, 1.0);
        assert!(!fs.lines[0].is_instrumental);
        assert_eq!(fs.lines[1].time_secs, 2.5);
        assert!(fs.lines[1].is_instrumental);
        assert_eq!(fs.lines[2].time_secs, 4.0);
        assert!(fs.lines[2].is_instrumental);
        assert_eq!(fs.lines[3].time_secs, 5.0);
        assert!(!fs.lines[3].is_instrumental);
    }

    #[test]
    fn test_from_lrc_str_multi_timestamp() {
        let lrc_content = "\
[00:01.00]First line
[00:03.00][00:06.00]Repeated chorus
[00:04.50]...
";
        let fs = FullscreenLyrics::from_lrc_str(lrc_content);
        // Should have 4 lines sorted: 1.0, 3.0, 4.5, 6.0
        assert_eq!(fs.len(), 4);
        assert_eq!(fs.lines[0].time_secs, 1.0);
        assert_eq!(fs.lines[0].text, "First line");
        assert_eq!(fs.lines[1].time_secs, 3.0);
        assert_eq!(fs.lines[1].text, "Repeated chorus");
        assert_eq!(fs.lines[2].time_secs, 4.5);
        assert_eq!(fs.lines[2].text, "...");
        assert!(fs.lines[2].is_instrumental);
        assert_eq!(fs.lines[3].time_secs, 6.0);
        assert_eq!(fs.lines[3].text, "Repeated chorus");
    }
}
