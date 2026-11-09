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
        let mut lines = Vec::new();
        let mut has_timestamp = false;
        let mut global_offset_ms: i64 = 0;

        for raw_line in content.lines() {
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

        Self {
            lines,
            is_synced: has_timestamp,
            source: source.to_string(),
        }
    }

    pub fn load_local(track_path: &Path) -> Option<Self> {
        // 1. Check sidecar .lrc file: path.with_extension("lrc")
        let lrc_path = track_path.with_extension("lrc");
        if lrc_path.exists() {
            if let Ok(content) = fs::read_to_string(&lrc_path) {
                return Some(Self::parse(&content, "Local .lrc"));
            }
        }

        // 2. Check embedded LYRICS / USLT tag
        if let Ok(tagged_file) = lofty::probe::Probe::open(track_path).and_then(|p| p.read()) {
            if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                if let Some(item) = tag.get(&lofty::tag::ItemKey::Lyrics) {
                    if let Some(lyrics_str) = item.value().text() {
                        return Some(Self::parse(lyrics_str, "Embedded Tag"));
                    }
                }
            }
        }

        None
    }
}

fn parse_timestamp(tag: &str) -> Option<u64> {
    // Supports formats: mm:ss.xx, mm:ss:xx, mm:ss
    let parts: Vec<&str> = tag.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let mins: u64 = parts[0].parse().ok()?;
    let rest = parts[1];

    let (secs, ms) = if let Some((s_str, ms_str)) = rest.split_once('.') {
        let s: u64 = s_str.parse().ok()?;
        let ms: u64 = match ms_str.len() {
            1 => ms_str.parse::<u64>().ok()? * 100,
            2 => ms_str.parse::<u64>().ok()? * 10,
            _ => ms_str.chars().take(3).collect::<String>().parse().ok()?,
        };
        (s, ms)
    } else if parts.len() == 3 {
        let s: u64 = rest.parse().ok()?;
        let ms: u64 = parts[2].parse::<u64>().ok()? * 10;
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
}
