//! Navigation Tab definitions for Eclipse.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavTab {
    // Library
    Home,
    NowPlaying,
    Songs,
    Artists,
    Albums,
    Genres,
    Folders,
    // Smart Playlists
    RecentlyPlayed,
    MostPlayed,
    RecentlyAdded,
    Favorites,
    NeverPlayed,
    RandomDiscovery,
    // Stream
    Spotify,
    YouTubeMusic,
    // Playlists
    Playlists,
    Playlist(i64),
    // Preferences & Info
    Settings,
    About,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_tab_variants_and_equality() {
        assert_eq!(NavTab::Home, NavTab::Home);
        assert_ne!(NavTab::Home, NavTab::Songs);
        assert_eq!(NavTab::Playlist(42), NavTab::Playlist(42));
        assert_ne!(NavTab::Playlist(1), NavTab::Playlist(2));
    }
}
