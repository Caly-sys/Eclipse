# TASK-R3 Handoff: "Save as Playlist" & "Find in My Library" Matcher with CJK Normalization

## 1. Overview
TASK-R3 implements the library matcher and playlist persistence engine for Eclipse (`src/streams/matcher.rs`).

When users import public stream tracks (from Spotify or other providers), they can:
1. **"Find in my library"**: Intelligently match imported stream tracks against local library audio files (`FLAC`, `WAV`, `MP3`, etc.) using CJK-safe unicode normalization, noise token stripping (`feat.`, `remix`, `remastered`, `live`), and strict duration tolerance (±5.0s). Matched tracks are marked with their local SQLite track ID and local file path so they play directly with lossless local fidelity.
2. **"Save as playlist"**: Persist imported tracklists into local SQLite playlists (`playlists` and `playlist_items` tables). Locally available tracks keep their existing local track IDs, while streaming-only tracks are cleanly inserted into `tracks` with `source = 'stream'` and `url = track.uri`.

All library matching and playlist saving operations execute asynchronously on background threads off the UI frame loop, guaranteeing fluid 60 FPS rendering.

---

## 2. Key Components & Implementation

### A. CJK-Safe Normalization & Noise Token Stripping (`normalize_string`)
Located in `src/streams/matcher.rs`:
- **Noise Token Stripping (`strip_noise_tokens`)**:
  - Bracketed noise tokens in `(...)`, `[...]`, `{...}`, `（...）`, `【...】`:
    - `(feat. ...)`, `[feat. ...]`, `(featuring ...)`, `[featuring ...]`, `(ft. ...)`, `[ft. ...]`
    - `(remix)`, `[remix]`, `(club remix)`, `(extended remix)`
    - `[remastered ...]`, `(remastered ...)`, `[remaster ...]`, `(2021 remaster)`
    - `(live ...)`, `[live ...]`, `(live at Wembley)`, `(Live)`
    - `bonus track`, `deluxe edition`, `radio edit`, `instrumental`, `acoustic version`
  - Unbracketed noise tokens:
    - `" ft. "`, `" feat. "`, `" featuring "`, `" ft "`, `" feat "` (e.g. `Song Title ft. Artist B` -> `Song Title`)
    - Trailing hyphenated noise: `" - Remastered"`, `" - 2021 Remaster"`, `" - Live"`, `" - Remix"`
  - Safeguard: If noise stripping leaves an empty string, the original normalized title is retained.
- **CJK-Safe Character Normalization**:
  - Fullwidth Latin (`\u{FF21}..\u{FF5A}`) and fullwidth digits (`\u{FF10}..\u{FF19}`) are converted to ASCII.
  - Ideographic fullwidth space `\u{3000}` is converted to standard ASCII space `' '`.
  - All CJK characters (Japanese Kanji, Hiragana, Katakana, Chinese Hanzi, Korean Hangul) are preserved using `c.is_alphanumeric()`.
  - Punctuation removal: ASCII punctuation and CJK punctuation (`、`, `。`, `・`, `「`, `」`, etc.) are stripped cleanly.
  - Separators (`-`, `/`, `_`, `\`, `|`) are replaced with space before whitespace collapsing so words are not conjoined (e.g., `hip-hop` -> `hip hop`).
  - Whitespace collapsing and trimming: consecutive spaces are collapsed into a single space `' '`.

### B. Fuzzy Matching Algorithm (`match_tracks_against_library`)
- **Duration Tolerance Boundary**:
  - Configurable tolerance (default `DEFAULT_DURATION_TOLERANCE_SECS = 5.0s`).
  - If both stream and local tracks have durations > 0, tracks with `duration_diff > 5.0s` are strictly rejected.
  - Tracks within `duration_diff <= 5.0s` receive duration closeness bonus points.
- **Title Matching**:
  - Exact match (`s_title == l_title`): 100 points.
  - Containment (`s_title.contains(&l_title)` or `l_title.contains(&s_title)`): 75 points.
- **Artist Matching**:
  - Exact artist match: 100 points.
  - Primary artist match (`primary_artist` extraction before `,`, `&`, `feat`, `/`, `x`): 85 points.
  - Artist / Album Artist containment: 70 points.
  - Rejection: If artist score < 70 and stream artist is non-empty, candidate is rejected.
- **Album & Lossless Bonuses**:
  - Album match: +10 to +15 bonus points.
  - Lossless audio (`is_lossless()` FLAC/WAV): +5 bonus points.
- **Scoring & Selection**:
  - Minimum match qualification threshold: 135.0 points.
  - The highest scoring candidate is selected; `StreamTrack` is updated with `local_track_id: Some(track.id)` and `local_path: Some(track.path)`.

### C. Playlist Persistence Engine (`save_imported_playlist`)
- Implemented in `src/streams/matcher.rs` and integrated with `Database`:
  1. Creates playlist via `db.create_playlist(name)`.
  2. For locally matched tracks (`local_track_id: Some(id)`): uses existing local track ID.
  3. For auto-matched tracks in library: automatically links to existing local track ID.
  4. For streaming-only tracks: inserts into `tracks` with:
     - `path`: `track.uri` (or fallback `stream:{provider}:{id}`)
     - `source`: `'stream'`
     - `url`: `track.uri`
     - `thumb`: `track.art_url`
     - `title`, `artist`, `album`, `duration`
  5. Inserts items into `playlist_items` via `db.add_track_to_playlist(playlist_id, track_id)`.
  6. Returns `PlaylistSaveResult { id, name, track_count }`.

### D. Streaming Metadata Model (`src/streams/provider.rs`)
- Added fields to `StreamTrack`:
  ```rust
  #[serde(default)]
  pub local_track_id: Option<i64>,
  #[serde(default)]
  pub local_path: Option<String>,
  ```
- Added helper methods:
  - `is_available_locally(&self) -> bool`: returns true if `local_track_id.is_some() || local_path.is_some()`.
  - `set_local_match(&mut self, track_id: i64, path: impl Into<String>)`.
  - `clear_local_match(&mut self)`.

### E. Controller & Worker Dispatch (`src/streams/controller.rs`)
- Added commands:
  - `StreamCommand::MatchLibrary(Vec<StreamTrack>)`
  - `StreamCommand::SaveImportedPlaylist { name: String, tracks: Vec<StreamTrack> }`
- Added events:
  - `StreamEvent::LibraryMatched(Vec<StreamTrack>)`
  - `StreamEvent::PlaylistSaved { id: i64, name: String, track_count: usize }`
- Added convenience dispatch methods on `StreamController`:
  - `controller.match_library(tracks)`
  - `controller.save_imported_playlist(name, tracks)`
- Handled asynchronously on background threads in `run_stream_worker` to keep the UI at constant 60 FPS.

---

## 3. Quality Gates Verification

### A. Compiler Check
```
cargo check
   Compiling eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.57s
```
Status: **0 errors, 0 warnings**.

### B. Clippy Linter
```
cargo clippy --all-targets -- -D warnings
    Checking eclipse v1.2.2 (C:\music\Eclipse)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.53s
```
Status: **0 warnings across all production and test targets**.

### C. Full Test Suite Execution
```
cargo test --all
running 148 tests
test audio::engine::tests::test_equal_power_crossfade_math ... ok
...
test streams::matcher::tests::test_cjk_string_matching ... ok
test streams::matcher::tests::test_duration_tolerance_boundary ... ok
test streams::matcher::tests::test_match_tracks_against_library_bulk ... ok
test streams::matcher::tests::test_noise_token_stripping ... ok
test streams::matcher::tests::test_save_as_playlist_integration_in_memory ... ok
test streams::provider::tests::test_stream_track_local_availability ... ok
test streams::controller::tests::test_stream_controller_matcher_and_playlist_commands ... ok
...
test result: ok. 148 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
```
- **Total Tests Passed**: **148 passed; 0 failed; 0 ignored** (7 new tests added, all passing).
- **Unit Tests Added**:
  1. `test_cjk_string_matching`: Verifies Japanese Kanji, Romaji, Hiragana, Katakana, Korean Hangul, fullwidth CJK forms, and candidate matching (`群青` by `YOASOBI`).
  2. `test_noise_token_stripping`: Verifies `(feat. ...)`, `[feat. ...]`, `ft. ...`, `(remix)`, `[remastered 2021]`, `(live at Wembley)`, `(Live)`, ` - Remastered`, and punctuation handling.
  3. `test_duration_tolerance_boundary`: Verifies diff = 4.9s and 5.0s match; diff = 5.1s and 8.0s reject.
  4. `test_save_as_playlist_integration_in_memory`: Tests end-to-end SQLite in-memory playlist persistence, local track ID preservation, streaming track insertion with `source = 'stream'`, and playlist querying.
  5. `test_match_tracks_against_library_bulk`: Verifies bulk matching with mixed local and streaming-only tracks.
  6. `test_stream_track_local_availability`: Verifies `is_available_locally`, `set_local_match`, and `clear_local_match`.
  7. `test_stream_controller_matcher_and_playlist_commands`: Verifies async dispatch and event round-trips for `MatchLibrary` and `SaveImportedPlaylist`.

---

## 4. Next Step Handoff
TASK-R3 is complete and marked `HANDOFF_READY` in `docs/streams/TASKS.md`.
Ready for:
- **`TASK-R4`** (`playback-agent`): YouTube Data API v3 matcher & official embedded playback with match cache (`src/streams/playback.rs`).
