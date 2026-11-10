# TASK-R4 Handoff: YouTube Data API v3 Lazy Matcher & Official Embedded Webview Player

## 1. Overview
TASK-R4 delivers the streaming playback engine for Eclipse (`src/streams/playback.rs`), enabling:
1. **Lazy YouTube Resolution**: When playing any stream track (imported from Spotify or other sources), matches the track via official YouTube Data API v3 search (`{artist} - {title}`).
2. **SQLite Match Caching**: Pairs (`spotify_id` -> `youtube_video_id`, title, channel, duration) are saved in the `stream_match_cache` table for instantaneous future playback.
3. **Queue Pre-resolution**: Lazily pre-resolves the *next* track in queue in the background.
4. **Alternative Match Selection**: Users can search for alternatives (`search_youtube_alternatives`) and override pairings (`select_alternative_match`), updating the SQLite cache.
5. **Quota Protection**: Identifies HTTP 403 `quotaExceeded` errors and surfaces clear user guidance: *"YouTube Data API daily quota exhausted (10,000 units/day). Please wait until tomorrow or provide your own API key in Settings > Streaming."*
6. **Official YouTube IFrame Player Hosting in Small Webview Window**: Uses `wry` (Windows WebView2 backend) and `tao` on a dedicated background thread to host `https://www.youtube.com/iframe_api` with `enablejsapi=1`.
7. **Bi-directional IPC**:
   - Outgoing: `player.playVideo()`, `player.pauseVideo()`, `player.seekTo(seconds, true)`, `player.setVolume(vol * 100)`, `player.loadVideoById(...)`.
   - Incoming via `window.ipc.postMessage`: `timeupdate` (updates `StreamPlaybackState` for the 60 FPS seekbar) and `statechange` (state 0 == ended advances queue).
8. **Strict Compliance**: ZERO `yt-dlp`, ZERO `ytmusicapi`, ZERO scraping, ZERO audio extraction or downloading.

---

## 2. Key Components & Implementation

### A. Lazy Matcher & SQLite Match Cache (`src/streams/playback.rs`, `src/library/db.rs`)
- **Query Generation**: `build_match_query(artist, title)` cleanly creates `{artist} - {title}` while safely handling whitespace and missing fields.
- **SQLite Schema**:
  ```sql
  CREATE TABLE IF NOT EXISTS stream_match_cache (
      spotify_id TEXT PRIMARY KEY,
      youtube_video_id TEXT NOT NULL,
      title TEXT NOT NULL,
      channel TEXT NOT NULL,
      duration_secs REAL NOT NULL,
      updated_at REAL NOT NULL
  );
  CREATE INDEX IF NOT EXISTS ix_stream_match_cache_spotify ON stream_match_cache(spotify_id);
  ```
- **CRUD Operations**: Added `get_stream_match`, `set_stream_match`, `delete_stream_match`, and `clear_stream_match_cache` to `Database`.
- **Resolution Flow**:
  1. Checks `stream_match_cache` for `track.id`.
  2. If missing, queries YouTube Data API v3 `search` endpoint (`videoCategoryId=10`, `type=video`).
  3. Enriches video details via `videos` endpoint to parse ISO 8601 durations and thumbnails.
  4. Saves match pairing in SQLite.
- **Background Prefetch**: `prefetch_next_track_match` asynchronously resolves and caches the next track in queue.
- **Alternative Selection**: `search_youtube_alternatives` returns alternative matches; `select_alternative_match` updates SQLite with the user's preferred match.

### B. Official YouTube IFrame Player in Dedicated Webview Window (`wry` + `tao`)
- **Webview Window**:
  - Small dedicated window (480x270, min 320x180) titled "Eclipse - Stream Player" on a separate background thread with `with_any_thread(true)`.
  - Lazy instantiation: window and webview are spawned on-demand when playback is initiated.
  - Close-safe: When user clicks the close button (`WindowEvent::CloseRequested`), the window is hidden (`window.set_visible(false)`) rather than destroying the process or interrupting audio.
  - Safe lifecycle: Uses `event_loop.run_return` for clean thread termination without `std::process::exit`.
- **Embedded HTML Template**:
  - Dark glass background (`#0d0a14`) matching Eclipse purple theme.
  - YouTube IFrame API initialization with `enablejsapi=1`, `autoplay=1`, `playsinline=1`, and `origin='https://www.youtube.com'`.
  - 250ms periodic `player.getCurrentTime()` / `player.getDuration()` time updates posted via `window.ipc.postMessage`.
  - Player state change listener: posts state changes, detecting track completion (`state == 0`).
- **Webview Commands**:
  - `LoadVideo { video_id, start_secs }`: executes `player.loadVideoById(...)`.
  - `Play`: executes `player.playVideo()`.
  - `Pause`: executes `player.pauseVideo()`.
  - `Seek(pos)`: executes `player.seekTo(pos, true)`.
  - `SetVolume(vol)`: executes `player.setVolume(vol * 100)`.
  - `SetVisible(bool)`: toggles window visibility.
  - `Shutdown`: cleanly exits event loop.

### C. Stream Controller & Queue Integration (`src/streams/controller.rs`, `src/ui/mod.rs`)
- **Commands**:
  - `StreamCommand::PlayStreamTrack(StreamTrack)`
  - `StreamCommand::PauseStream`
  - `StreamCommand::ResumeStream`
  - `StreamCommand::SeekStream(f64)`
  - `StreamCommand::SetStreamVolume(f32)`
  - `StreamCommand::SelectAlternativeMatch { spotify_id, alternative }`
  - `StreamCommand::SearchAlternatives { spotify_id, query }`
  - `StreamCommand::PrefetchNextTrack(StreamTrack)`
- **Events**:
  - `StreamEvent::StreamTrackEnded`
  - `StreamEvent::AlternativesLoaded { spotify_id, alternatives }`
  - `StreamEvent::AlternativeSelected { spotify_id, matched }`
- **Stream Queue**:
  - Added `stream_queue: Vec<StreamTrack>` and `stream_queue_index: Option<usize>` in `StreamController`.
  - Helper methods: `load_stream_queue`, `has_next_in_queue`, `play_next_in_queue`, `play_prev_in_queue`.
  - Controller `next(&prov_id)` and `prev(&prov_id)` automatically navigate the stream queue.
  - In `src/ui/mod.rs`, `StreamEvent::StreamTrackEnded` automatically advances to the next track in `stream_queue`.

---

## 3. Quality Gates Verification

### A. Compiler Check
```
cargo check
   Checking eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.22s
```
Status: **0 errors, 0 warnings**.

### B. Clippy Linter
```
cargo clippy --all-targets -- -D warnings
   Checking eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.96s
```
Status: **0 warnings across all production and test targets**.

### C. Full Test Suite Execution
```
cargo test --all
running 156 tests
test library::db::tests::test_stream_match_cache_crud ... ok
test streams::controller::tests::test_stream_controller_playback_and_queue_commands ... ok
test streams::playback::tests::test_match_query_generation ... ok
test streams::playback::tests::test_quota_error_detection ... ok
test streams::playback::tests::test_sqlite_match_cache_save_and_retrieval ... ok
test streams::playback::tests::test_html_embed_template_rendering ... ok
test streams::playback::tests::test_ipc_message_deserialization ... ok
test streams::playback::tests::test_alternative_match_model_conversion ... ok
...
test result: ok. 156 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s
```
- **Total Tests Passed**: **156 passed; 0 failed; 0 ignored** (8 new tests added, all passing).
- **Zero Lossless Regressions**: 100% of the 102 original lossless player tests continue passing cleanly.

---

## 4. Next Step Handoff
TASK-R4 is complete and marked `HANDOFF_READY` in `docs/streams/TASKS.md`.
Ready for:
- **`TASK-R5`** (`stream-ui-agent`): Rebuilt Spotify Importer UI matching reference image (`docs/reference/stream-spotify.png`), track table, Play all, Shuffle, and playerbar integration.
