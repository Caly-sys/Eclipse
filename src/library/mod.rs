pub mod art_fetcher;
pub mod db;
pub mod genre_fetcher;
pub mod model;
pub mod scanner;

pub use art_fetcher::{fetch_missing_cover_art, queue_art_backfill};
pub use db::{Database, LibraryDatabase};
pub use genre_fetcher::{
    clean_artist, clean_title, embed_genre_in_file, fetch_track_genre,
    fetch_track_genre_with_album, get_genre_fetch_progress, is_genre_fetch_running,
    is_missing_or_unknown_genre, queue_genre_backfill, start_genre_backfill,
};
pub use model::{format_time_secs, Playlist, Track};
pub use scanner::{load_cached_album_art, scan_directories};
