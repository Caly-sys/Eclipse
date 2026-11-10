# TASK-R2 Handoff: Zero-Login Public Spotify Link Metadata Importer

## 1. Overview
TASK-R2 delivers the zero-login, zero-setup public Spotify metadata importer for Eclipse (`src/streams/spotify_public.rs`).

Users can paste any public Spotify URL (`open.spotify.com/track/...`, `/album/...`, `/playlist/...`) or `spotify:` URI. Eclipse extracts public track and album metadata (title, artist, album, duration, artwork URL) directly from public Spotify embed pages (`open.spotify.com/embed/...`) and the official Spotify oEmbed endpoint (`open.spotify.com/oembed`), with zero user authentication, zero Spotify developer apps, and zero client ID configuration.

All imported metadata is cached locally in SQLite (`stream_cache` table) so re-importing the same link avoids redundant network requests, and all I/O is dispatched asynchronously off the UI thread to ensure consistent 60 FPS rendering.

---

## 2. Key Components & Implementation

### A. Trait & Result Types (`src/streams/spotify_public.rs`)
- **`SpotifyPublicMetadataFetcher`**:
  ```rust
  pub trait SpotifyPublicMetadataFetcher: Send + Sync {
      fn fetch_metadata(&self, input: &str) -> Result<SpotifyImportResult, String>;
  }
  ```
- **`SpotifyResourceType`**:
  `Track`, `Album`, `Playlist` with string conversions (`as_str()`, `display_name()`).
- **`SpotifyImportResult`**:
  Standardized result holding:
  - `resource_type: SpotifyResourceType`
  - `id: String`
  - `title: String`
  - `subtitle: Option<String>` (artist or playlist owner)
  - `cover_url: Option<String>`
  - `tracks: Vec<StreamTrack>` (mapped with `provider = "spotify"` and canonical URI `spotify:track:{id}`)
  - `total_tracks: usize`
  - `is_truncated: bool`
  - `truncation_message: Option<String>`
  - `original_url: String`

### B. URL & URI Parser (`parse_spotify_link`)
- Supports:
  - Standard track URLs: `https://open.spotify.com/track/{id}`
  - Standard album URLs: `https://open.spotify.com/album/{id}`
  - Standard playlist URLs: `https://open.spotify.com/playlist/{id}`
  - Embed URLs: `https://open.spotify.com/embed/{track|album|playlist}/{id}`
  - Internationalized URLs: `https://open.spotify.com/intl-{lang}/{track|album|playlist}/{id}`
  - Legacy user playlists: `open.spotify.com/user/{user}/playlist/{id}`
  - Spotify URIs: `spotify:track:{id}`, `spotify:album:{id}`, `spotify:playlist:{id}`, `spotify:user:{user}:playlist:{id}`
  - Bare domain inputs without `https://` prefix (e.g. `open.spotify.com/track/{id}`)
- Preserves clean 22-character alphanumeric Spotify IDs across query strings (e.g. `?si=...`), fragments (`#...`), and trailing slashes.
- Rejects non-Spotify URLs, empty strings, and unsupported types (e.g. `artist`) with clear, user-friendly error messages.

### C. Multi-Tier Public Extraction Engine
1. **Primary: Spotify Embed JSON Extraction**:
   - Fetches `https://open.spotify.com/embed/{type}/{id}` using `ureq`.
   - Parses `<script id="__NEXT_DATA__" type="application/json">` without panics.
   - Extracts track lists, individual track titles, artists, duration (converted from milliseconds to seconds), and high-resolution album cover art.
2. **Fallback 1: HTML Open Graph Parsing**:
   - Parses `<meta property="og:title">`, `<meta property="og:description">`, `<meta property="og:image">`.
   - Cleans Spotify site branding and unescapes standard HTML entities.
3. **Fallback 2: Spotify Official oEmbed Endpoint**:
   - Queries `https://open.spotify.com/oembed?url={original_url}` for confirmed title and thumbnail artwork URL.

### D. Truncation & Edge-Case Resilience
- **Truncation**: For playlists with >= 100 tracks (where Spotify embed payloads cap), calculates `total_tracks` and marks `is_truncated = true` with `truncation_message = Some("Showing first 100 tracks")` or `"Showing first 100 of N tracks"`.
- **404 / Private links**: Returns friendly `"This Spotify link could not be found or is set to private. Please ensure the link is public."` (detected via HTTP 404, JSON `status: 404`, or `"Page not found"`).
- **Rate limits (429)**: Returns friendly `"Spotify rate limit encountered (HTTP 429). Please try again shortly."`.
- **Offline / Transport errors**: Returns clear error explaining connection failure.
- **Layout changes**: Fails cleanly with descriptive error and zero panics.

### E. SQLite Caching (`src/library/db.rs`)
- Added `stream_cache` table to SQLite schema with `(key PRIMARY KEY, provider, resource_type, data, updated_at)`.
- Added CRUD methods on `Database`: `get_stream_cache`, `set_stream_cache`, `delete_stream_cache`, `clear_stream_cache`.
- Canonical Spotify URI (`spotify:{type}:{id}`) is used as the primary cache key. Repeated imports of the same link bypass network I/O entirely.

### F. Controller & Background Worker Integration (`src/streams/controller.rs`)
- Added `StreamCommand::ImportSpotifyLink(String)` to `StreamCommand`.
- Added `StreamEvent::SpotifyImportComplete(Result<SpotifyImportResult, String>)` to `StreamEvent`.
- Added `StreamController::import_spotify_link(&mut self, url: &str)` convenience method.
- Network requests are spawned off the `eclipse-stream-worker` background thread to guarantee 60 FPS UI rendering without stutter.
- Added `last_spotify_import` state tracking in `StreamController`.

---

## 3. Quality Gates Verification

### A. Compiler Check
```
cargo check
   Compiling eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.08s
```
Status: **0 errors, 0 warnings**.

### B. Clippy Linter
```
cargo clippy --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.32s
```
Status: **0 warnings across all targets**.

### C. Full Test Suite Execution
```
cargo test --all
test result: ok. 141 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
```
- **Total Tests Passed**: **141 passed; 0 failed; 0 ignored** (14 new tests added, all passing).
- **New Unit Tests**:
  - `test_parse_spotify_track_urls_and_queries`: Verifies standard, embed, international, query string, and trailing slash track URLs.
  - `test_parse_spotify_album_urls`: Verifies album URLs.
  - `test_parse_spotify_playlist_urls`: Verifies playlist and legacy user playlist URLs.
  - `test_parse_spotify_uris`: Verifies `spotify:track:`, `spotify:album:`, `spotify:playlist:`, and `spotify:user:playlist:` URIs.
  - `test_parse_invalid_links_and_errors`: Verifies empty, non-Spotify, and unsupported resource rejection.
  - `test_extract_embed_json_and_track_metadata`: Verifies full JSON track extraction and duration conversion.
  - `test_extract_album_with_multiple_tracks`: Verifies multi-track album extraction and album name/art inheritance.
  - `test_playlist_truncation_detection`: Verifies truncation flags and message generation for playlists with >= 100 tracks.
  - `test_embed_404_detection`: Verifies friendly error message for missing/private links.
  - `test_open_graph_fallback_extraction`: Verifies Open Graph meta tag extraction.
  - `test_oembed_json_parsing`: Verifies supplementary oEmbed parsing.
  - `test_sqlite_cache_integration`: Verifies cache hit bypasses network.
  - `test_stream_cache_crud`: Verifies SQLite CRUD operations for `stream_cache`.
  - `test_stream_controller_import_spotify_link_dispatch`: Verifies controller dispatch and event handling on background worker.

---

## 4. Next Step Handoff
TASK-R2 is complete and marked `HANDOFF_READY` in `docs/streams/TASKS.md`.
Ready for:
- **`TASK-R3`** (`library-match-agent`): Implement `src/streams/matcher.rs` ("Save as playlist" and "Find in my library" matcher with CJK / fuzzy normalization and duration tolerance).
