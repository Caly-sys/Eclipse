use lofty::file::TaggedFileExt;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub is_synced: bool,
    pub source: String,
}

impl Lyrics {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn current_index(&self, current_ms: u64) -> Option<usize> {
        if self.lines.is_empty() || !self.is_synced {
            return None;
        }

        let mut idx = 0;
        let mut found = false;

        for (i, line) in self.lines.iter().enumerate() {
            if current_ms >= line.timestamp_ms {
                idx = i;
                found = true;
            } else {
                break;
            }
        }

        if found {
            Some(idx)
        } else {
            None
        }
    }

    pub fn parse(content: &str, source: &str) -> Self {
        let trimmed_content = content.trim();
        let mut lines = Vec::new();
        let mut has_timestamp = false;
        let mut global_offset_ms: i64 = 0;

        for raw_line in trimmed_content.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }

            // Scan for all bracketed tags at the beginning of the line
            let mut remaining = line;
            let mut line_timestamps = Vec::new();
            let mut is_header = false;

            while remaining.starts_with('[') {
                if let Some(close_idx) = remaining.find(']') {
                    let tag = remaining[1..close_idx].trim();
                    let tag_lower = tag.to_lowercase();

                    if let Some(offset_str) = tag_lower.strip_prefix("offset:") {
                        if let Ok(off) = offset_str.trim().parse::<i64>() {
                            global_offset_ms = off;
                        }
                        is_header = true;
                        remaining = remaining[close_idx + 1..].trim();
                        break;
                    } else if is_metadata_tag(tag) {
                        is_header = true;
                        remaining = remaining[close_idx + 1..].trim();
                        break;
                    } else if let Some(ts_ms) = parse_timestamp(tag) {
                        line_timestamps.push(ts_ms);
                        remaining = remaining[close_idx + 1..].trim();
                    } else {
                        // Unrecognized bracket tag
                        is_header = true;
                        break;
                    }
                } else {
                    break;
                }
            }

            if is_header {
                continue;
            }

            if !line_timestamps.is_empty() {
                has_timestamp = true;
                let text = remaining.trim().to_string();
                for ts in line_timestamps {
                    lines.push(LyricLine {
                        timestamp_ms: ts,
                        text: text.clone(),
                    });
                }
            } else if !line.starts_with('[') {
                // Regular unsynced line (plain text)
                lines.push(LyricLine {
                    timestamp_ms: 0,
                    text: line.to_string(),
                });
            }
        }

        if has_timestamp {
            // Apply global offset if present
            if global_offset_ms != 0 {
                for line in &mut lines {
                    let new_ts = (line.timestamp_ms as i64 + global_offset_ms).max(0);
                    line.timestamp_ms = new_ts as u64;
                }
            }

            lines.sort_by_key(|l| l.timestamp_ms);
        }

        // Strip trailing blank lines
        while lines.last().is_some_and(|l| l.text.trim().is_empty()) {
            lines.pop();
        }

        Self {
            lines,
            is_synced: has_timestamp,
            source: source.to_string(),
        }
    }

    pub fn load_local(track_path: &Path) -> Option<Self> {
        Self::load_with_cache(track_path)
    }

    pub fn load_with_cache(track_path: &Path) -> Option<Self> {
        // 1. Check sidecar .lrc file: path.with_extension("lrc")
        let lrc_path = track_path.with_extension("lrc");
        if lrc_path.exists() {
            if let Ok(content) = fs::read_to_string(&lrc_path) {
                if !content.trim().is_empty() {
                    return Some(Self::parse(&content, "Local .lrc"));
                }
            }
        }

        // 2. Check local lyrics cache folder (%APPDATA%\Eclipse\lyrics_cache)
        if let Some(content) = crate::lyrics::lrclib::load_from_cache(track_path) {
            if !content.trim().is_empty() {
                return Some(Self::parse(&content, "Lyrics Cache"));
            }
        }

        // 3. Check embedded LYRICS / USLT tag
        if let Ok(tagged_file) = lofty::probe::Probe::open(track_path).and_then(|p| p.read()) {
            if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                if let Some(item) = tag.get(&lofty::tag::ItemKey::Lyrics) {
                    if let Some(lyrics_str) = item.value().text() {
                        if !lyrics_str.trim().is_empty() {
                            return Some(Self::parse(lyrics_str, "Embedded Tag"));
                        }
                    }
                }
            }
        }

        None
    }
}

fn parse_timestamp(tag: &str) -> Option<u64> {
    let clean = tag.trim();
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let mins: u64 = parts[0].trim().parse().ok()?;
    let rest = parts[1].trim();

    let (secs, ms) = if let Some((s_str, ms_str)) = rest.split_once('.') {
        let s: u64 = s_str.trim().parse().ok()?;
        let digits: String = ms_str.chars().filter(|c| c.is_ascii_digit()).collect();
        let ms: u64 = match digits.len() {
            0 => 0,
            1 => digits.parse::<u64>().ok()? * 100,
            2 => digits.parse::<u64>().ok()? * 10,
            3 => digits.parse::<u64>().ok()?,
            _ => digits[..3].parse::<u64>().ok()?,
        };
        (s, ms)
    } else if parts.len() == 3 {
        let s: u64 = rest.parse().ok()?;
        let cs_digits: String = parts[2].trim().chars().filter(|c| c.is_ascii_digit()).collect();
        let ms: u64 = match cs_digits.len() {
            0 => 0,
            1 => cs_digits.parse::<u64>().ok()? * 100,
            2 => cs_digits.parse::<u64>().ok()? * 10,
            3 => cs_digits.parse::<u64>().ok()?,
            _ => cs_digits[..3].parse::<u64>().ok()?,
        };
        (s, ms)
    } else {
        let s: u64 = rest.parse().ok()?;
        (s, 0)
    };

    Some(mins * 60 * 1000 + secs * 1000 + ms)
}

fn is_metadata_tag(tag: &str) -> bool {
    let lower = tag.to_lowercase();
    lower.starts_with("ti:")
        || lower.starts_with("ar:")
        || lower.starts_with("al:")
        || lower.starts_with("by:")
        || lower.starts_with("offset:")
        || lower.starts_with("length:")
        || lower.starts_with("re:")
        || lower.starts_with("ve:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_headers_ignored() {
        let content = r#"
[ti:Kimi wa Rock wo Kikanai]
[ar:Aimyon]
[al:Excitement of Youth]
[by:SpotiFlac]
[offset:500]
[00:12.50]Line one lyrics
[00:18.00]Line two lyrics
"#;
        let lrc = Lyrics::parse(content, "test");
        assert!(lrc.is_synced);
        assert_eq!(lrc.lines.len(), 2);
        assert_eq!(lrc.lines[0].text, "Line one lyrics");
        // Offset +500 applied: 12500 + 500 = 13000
        assert_eq!(lrc.lines[0].timestamp_ms, 13000);
        assert_eq!(lrc.lines[1].text, "Line two lyrics");
        assert_eq!(lrc.lines[1].timestamp_ms, 18500);
    }

    #[test]
    fn test_parse_unsynced_lyrics() {
        let content = "Just plain text\nNo timestamps here\nAnother line";
        let lrc = Lyrics::parse(content, "test");
        assert!(!lrc.is_synced);
        assert_eq!(lrc.lines.len(), 3);
        assert_eq!(lrc.lines[0].text, "Just plain text");
    }

    #[test]
    fn test_parse_timestamp_formats() {
        // [mm:ss.xx]
        let content_xx = "[01:23.45] Two-digit centiseconds";
        let lrc_xx = Lyrics::parse(content_xx, "test");
        assert_eq!(lrc_xx.lines[0].timestamp_ms, 83450);

        // [mm:ss.xxx]
        let content_xxx = "[01:23.456] Three-digit milliseconds";
        let lrc_xxx = Lyrics::parse(content_xxx, "test");
        assert_eq!(lrc_xxx.lines[0].timestamp_ms, 83456);

        // [mm:ss]
        let content_ss = "[01:23] Seconds only";
        let lrc_ss = Lyrics::parse(content_ss, "test");
        assert_eq!(lrc_ss.lines[0].timestamp_ms, 83000);

        // [mm:ss:xx] (colon centiseconds)
        let content_colon = "[01:23:45] Colon format";
        let lrc_colon = Lyrics::parse(content_colon, "test");
        assert_eq!(lrc_colon.lines[0].timestamp_ms, 83450);
    }

    #[test]
    fn test_parse_chronological_sort() {
        let content = r#"
[00:30.00] Third line
[00:10.00] First line
[00:20.00] Second line
"#;
        let lrc = Lyrics::parse(content, "test");
        assert_eq!(lrc.lines.len(), 3);
        assert_eq!(lrc.lines[0].text, "First line");
        assert_eq!(lrc.lines[0].timestamp_ms, 10000);
        assert_eq!(lrc.lines[1].text, "Second line");
        assert_eq!(lrc.lines[1].timestamp_ms, 20000);
        assert_eq!(lrc.lines[2].text, "Third line");
        assert_eq!(lrc.lines[2].timestamp_ms, 30000);
    }

    #[test]
    fn test_parse_strips_trailing_blank_lines() {
        let content = "[00:10.00] First line\n[00:20.00] Second line\n[00:30.00]   \n\n   \n";
        let lrc = Lyrics::parse(content, "test");
        assert_eq!(lrc.lines.len(), 2);
        assert_eq!(lrc.lines[1].text, "Second line");
    }
}
