pub mod lrc;
pub mod lrclib;

pub use lrc::{LyricLine, Lyrics};
pub use lrclib::{fetch_online, fetch_online_with_fallback, save_to_cache, search_lrclib_manual, LrclibRecord};
