# Eclipse Stream Rebuild Plan & Architectural Spec

## 1. Executive Summary
The previous streaming implementation introduced an unnecessary Spotify Developer App requirement (OAuth 2.0 PKCE, client ID configuration, local HTTP redirect loopback server on `127.0.0.1:8888`, and Spotify Connect device player controls).

This rebuild completely removes all developer app / OAuth / Connect requirements. As shown in the Umbra reference interface (`docs/reference/stream-spotify.png`), the new streaming system is a **zero-login, zero-setup public link importer**:
1. Users paste any public Spotify URL (`open.spotify.com/track/...`, `/album/...`, `/playlist/...`) or `spotify:` URI.
2. Eclipse extracts public track and album metadata (title, artist, album, duration, artwork URL) directly from public Spotify embed & oEmbed data.
3. Users can **Save as playlist**, **Find in my library** (match against local files), or **Stream / Play all** (matched from YouTube official embedded playback via official YouTube Data API v3).
4. No user ever needs to create a Spotify developer account, client ID, or grant permissions.

---

## 2. Deletion vs. Retention Inventory

### What Will Be Deleted:
- `src/streams/spotify.rs`: Entire RFC 7636 PKCE OAuth implementation, code verifier/challenge logic, local loopback HTTP server (`listen_for_auth_code` on `127.0.0.1:8888`), Spotify Connect Web API client, and Spotify device polling.
- Spotify OAuth credential handling in `src/streams/credentials.rs`: Delete `save_spotify_tokens`, `get_spotify_access_token`, `get_spotify_refresh_token`, `clear_spotify_credentials`. Keep clean migration that clears old keyring entries on first launch.
- Settings & UI Connect screens: Delete "Connect with Spotify" setup view, Client ID input cards, Developer Dashboard link buttons, and Connect device pickers in `src/ui/views/streams.rs` and `src/ui/views/settings.rs`.
- Dead tests: Delete all Spotify PKCE, OAuth, and Connect tests.

### What Will Be Kept & Reused:
- Core Provider Abstraction (`src/streams/provider.rs`):
  - `PlaybackSource` enum (`Local | Stream(String)`).
  - `StreamTrack` metadata model (`id`, `title`, `artist`, `album`, `duration_secs`, `art_url`, `uri`, `provider`).
  - `StreamPlaylist` metadata model (`id`, `name`, `track_count`, `description`, `art_url`).
  - `StreamPlaybackState` (`is_playing`, `current_track`, `position_secs`, `duration_secs`, `volume`).
- YouTube Official Integration (`src/streams/youtube.rs`):
  - Keep the official YouTube Data API v3 client for metadata and track matching.
  - Keep the official embedded browser playback launcher via `webbrowser::open` (ADR-003).
  - Keep ISO 8601 duration parser and HTML entity unescaping.
  - Keep YouTube API key storage in Windows Credential Manager (`keyring`).
- Audio Engine Decoupling (`src/audio/mod.rs`):
  - `playback_source` routing and mutual exclusivity (pausing local Rodio audio when streaming).
  - LRCLIB lyrics synchronization by title and artist.
- Theme tokens & vector icons:
  - Centralized purple glass frames in `src/ui/theme.rs`.
  - Crisp vector icons in `src/ui/icons.rs` (`Icon::Spotify`, `Icon::YouTube`, `Icon::Play`, etc.).

---

## 3. New Modules & Architecture

### A. `src/streams/spotify_public.rs` (Link Importer)
- Single self-contained module behind a trait:
  ```rust
  pub trait SpotifyPublicMetadataFetcher: Send + Sync {
      fn fetch_metadata(&self, input: &str) -> Result<SpotifyImportResult, String>;
  }
  ```
- Accepts:
  - `https://open.spotify.com/track/{id}`
  - `https://open.spotify.com/album/{id}`
  - `https://open.spotify.com/playlist/{id}`
  - `spotify:track:{id}`, `spotify:album:{id}`, `spotify:playlist:{id}`
- Scraping-free official public metadata extraction:
  - Fetches public embed page `https://open.spotify.com/embed/{type}/{id}` and parses JSON resource data (e.g. `__NEXT_DATA__` or embed props).
  - Falls back to Spotify oEmbed endpoint `https://open.spotify.com/oembed?url=...` for single tracks or album summaries.
- Handles pagination / partial lists ("showing first N tracks"), private links, 404s, and network outages.
- Caches imported metadata in local SQLite database (`db.rs`).

### B. `src/streams/matcher.rs` (Library Matcher)
- Implements "Find in my library":
  - Matches imported stream tracks against indexed local tracks in SQLite (`db.get_all_tracks()`).
  - Normalizes titles and artists (case folding, punctuation stripping, CJK unicode normalization, removing `(feat. ...)` / `[Remix]`).
  - Duration tolerance matching (within ±3–5 seconds).
  - Tags matching tracks with local file path so they can play with lossless local fidelity.

### C. `src/streams/playback.rs` (YouTube Match & Stream Dispatcher)
- When user plays a track not present locally:
  - Searches YouTube Data API v3 for `{artist} - {title}`.
  - Caches the matched video ID locally in SQLite so future plays are instant.
  - Allows user to pick alternative YouTube match if needed.
  - Plays through official embedded player (default browser / ADR-003).

### D. `src/ui/views/streams.rs` (Rebuilt Importer UI)
- Matches reference screenshot `docs/reference/stream-spotify.png` identically:
  - **Header Banner**: Spotify glass card, `STREAM` micro-tag, `Spotify` title, and subtitle:
    *"Paste a public Spotify track, album or playlist link. Save it as a playlist, stream it (matched from YouTube — Spotify doesn't allow third-party streaming) or download it."*
  - **Search / Paste Bar**: Link input field + `Enter` pill + white `🔍 Import` pill button.
  - **Action Row**: `▶ Play all`, `🔀 Shuffle`, `🎵 Save all as playlist`, `🔍 Find in my library`, and right-aligned `Recently streamed`.
  - **Multi-Column Track Table**: `#`, 40x40px album art thumbnail, Title, `SP` pill badge + Artist, Album, Added timestamp, Local / Stream status, Duration.
  - Smooth independent scrolling with `.id_salt`.
  - Seamless integration into Player Bar and Now Playing lyrics.
