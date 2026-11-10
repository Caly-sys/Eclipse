# TASK-R5 Handoff: Spotify Public Link Importer UI, Stream Transport UI & Playerbar Integration

## 1. Overview
TASK-R5 delivers the redesigned stream user interface for Eclipse (`src/ui/views/streams.rs`, `src/ui/icons.rs`, `src/ui/art.rs`, `src/ui/mod.rs`), fulfilling the full Spotify public link import design specification from `docs/reference/stream-spotify.png`.

Key capabilities implemented:
1. **Spotify Public Link Importer View**:
   - Glass-frosted header card matching `docs/reference/stream-spotify.png` with a 68x68 dark rounded square, Spotify green vector icon, `STREAM` micro-tag, `Spotify` header, and subtitle detailing streaming, matching, and downloading.
   - Input bar with search icon, hint text, keyboard `ENTER` pill badge, and white pill `Import` button.
   - Full action bar: `Play all`, `Shuffle`, `Save all as playlist`, `Find in my library`, and `Recently streamed` (linking to navigation history).
   - Multi-column table layout: `#` (or animated play indicator on hover/active), `TITLE` (40x40 thumbnail, bold track title, official Spotify green `SP` badge, `LOCAL` lossless badge when matched, artist), `ALBUM`, `ADDED`, `PLAYS` (pure vector `Icon::Download`), and `TIME`.
2. **Pure Vector Iconography**:
   - Zero Unicode emojis across the codebase.
   - Added `Icon::Download` vector icon in `src/ui/icons.rs` with clean geometric lines (vertical shaft, chevron arrow, base tray).
3. **Remote Cover Art Loading**:
   - Extended `AlbumArtManager` in `src/ui/art.rs` to fetch remote cover art via HTTP/HTTPS in the dedicated background thread with 4-second timeout and 512x512 downsampling to preserve GPU memory without stalling the 60 FPS UI loop.
4. **Seamless Local Lossless vs. Stream Playback Routing**:
   - Clicking a track row automatically checks `track.is_available_locally()`. If matched, Eclipse immediately plays the high-fidelity lossless file through the native audio engine (`PlayerController`).
   - If stream-only, it routes playback to the official embedded player via `StreamController::play_stream_track`, while prefetching the next track in the queue.
5. **State Handling**:
   - Handles empty states with quick-start Spotify public link suggestions.
   - Loading states with glowing glass indicators.
   - Error states with actionable retry buttons.
   - Truncation warning banners for large playlists exceeding public embed limits (first 100 tracks).
6. **YouTube Music View**:
   - Complementary YouTube Music streaming view with API key configuration notice, search bar, and integrated playback controls.

---

## 2. Key Components & Implementation

### A. Spotify Importer UI (`src/ui/views/streams.rs`)
- **Header Card & Input Bar**:
  - Encapsulated within `theme.glass_frame(14.0)` with dark purple glass aesthetic.
  - Implements the exact hierarchy from `docs/reference/stream-spotify.png`:
    - 68x68 square card with `Icon::Spotify` in `#1ED760`.
    - `STREAM` tag in muted purple, `Spotify` headline in 29pt strong text.
    - Full descriptive subtitle: *"Paste a public Spotify track, album or playlist link. Save it as a playlist, stream it (matched from YouTube — Spotify doesn't allow third-party streaming) or download it."*
    - Rounded 12px pill input bar with `Icon::Search`, text input field, `ENTER` pill badge, and white pill `Import` button. Supports keyboard <kbd>Enter</kbd> trigger.
- **Action Buttons Bar**:
  - Displays imported playlist/album title, subtitle, and green `TRACKS` badge count.
  - `Play all`: Loads imported tracks into the streaming queue at index 0.
  - `Shuffle`: Randomizes tracks and loads queue.
  - `Save all as playlist`: Dispatches `stream_controller.save_imported_playlist(...)` to store tracks in SQLite.
  - `Find in my library`: Dispatches `stream_controller.match_library(...)` to identify local lossless matches.
  - `Recently streamed`: Navigates directly to `NavTab::RecentlyPlayed`.
- **Tracklist Table**:
  - Proportional column widths matching design reference:
    - `#`: Row index or hover play icon / currently-playing animated icon.
    - `TITLE`: 40x40 art thumbnail, bold title, `SP` pill badge (`#1ED760`), `LOCAL` format badge (if matched), artist name.
    - `ALBUM`: Display album name with clean truncation.
    - `ADDED`: Relative added indicator (`1 d`).
    - `PLAYS`: Vector download icon (`Icon::Download`).
    - `TIME`: Formatted duration (`mm:ss`).
  - Row interaction: single-click selects row; click or double-click triggers instant playback (local lossless or stream via webview).

### B. Pure Vector Iconography (`src/ui/icons.rs`)
- Added `Icon::Download`:
  ```rust
  Icon::Download => {
      // Arrow pointing down: vertical shaft + chevron
      painter.line_segment([p(12.0, 4.0), p(12.0, 14.5)], stroke);
      painter.line_segment([p(7.5, 10.5), p(12.0, 15.0)], stroke);
      painter.line_segment([p(16.5, 10.5), p(12.0, 15.0)], stroke);
      // Tray / horizontal base
      painter.line_segment([p(5.0, 17.0), p(5.0, 19.5)], stroke);
      painter.line_segment([p(5.0, 19.5), p(19.0, 19.5)], stroke);
      painter.line_segment([p(19.0, 19.5), p(19.0, 17.0)], stroke);
  }
  ```
- Tested against `tests/verify_no_emojis.py`: 100% vector painting, 0 Unicode emojis across all `.rs` files.

### C. Async Remote Cover Art Loading (`src/ui/art.rs`)
- Background worker thread now detects URLs prefixed with `http://` or `https://`:
  - Fetches image bytes via `ureq::get` with a 4-second timeout.
  - Decodes into RGBA via `image::load_from_memory`.
  - Resizes to max 512x512 thumbnail.
  - Transmits to UI thread through channel for texture handle caching.
  - Zero UI thread latency or frame drops during thumbnail retrieval.

### D. Integration with Main UI & App Controller (`src/ui/mod.rs`, `src/streams/controller.rs`)
- Handled `StreamEvent::PlaylistSaved`: displays toast notification and reloads library playlists.
- Handled `StreamEvent::LibraryMatched`: displays count of matched tracks and updates the active import in place.
- Handled `StreamEvent::SpotifyImportComplete`: updates UI status with result count.
- Updated `render_spotify_view` and `render_youtube_view` call sites with `&mut self.art_manager` and `&mut on_navigate`.
- Derived `Default` on `Track` (`src/library/model.rs`) for clean fallback conversion when playing matched local files.

---

## 3. Quality Gates Verification

### A. Compiler Check
```
cargo check
    Checking eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.66s
```
Status: **0 errors, 0 warnings**.

### B. Clippy Linter
```
cargo clippy --all-targets -- -D warnings
    Checking eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.78s
```
Status: **0 warnings, clean**.

### C. Test Suite
```
cargo test --all
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.77s
     Running unittests src\main.rs (target\debug\deps\eclipse-f70f28ed0ce93990.exe)

running 156 tests
test ... ok
...
test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s
```
Status: **156/156 tests passed (100% passing clean)**.

### D. Zero Emoji Enforcement
```
python tests/verify_no_emojis.py
Scanned 45 Rust files in C:\music\Eclipse\src.
PASS: Verified 0 Unicode emojis across all .rs files in src/.
```
Status: **PASS (0 emojis detected)**.

---

## 4. Deliverables & Handoff Summary
- `src/ui/views/streams.rs`: Fully implemented Spotify Public Link Importer and YouTube views matching reference design.
- `src/ui/icons.rs`: Added `Icon::Download` vector icon.
- `src/ui/art.rs`: Remote thumbnail loading in background worker.
- `src/ui/mod.rs`: Stream event handlers and view wiring.
- `src/streams/controller.rs`: Stream queue navigation and match synchronization.
- `src/library/model.rs`: Derived `Default` for `Track`.
- `docs/streams/TASKS.md`: Updated `TASK-R5` status to `HANDOFF_READY`.

Ready for `qa-reviewer` on **TASK-R6** (End-to-End Regression Audit & Upgrade Migration Verification).
