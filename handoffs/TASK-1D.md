# Handoff: TASK-1D - Target Screens, 3-Column Now Playing, Audio Analyzer, Smart Playlists & Navigation

## ✦ 1. Task Summary
- **Task ID**: `TASK-1D`
- **Agent**: `feature`
- **Status**: `HANDOFF_READY`
- **Reference**: `docs/reference/target-umbra.png`, `DESIGN.md`, `PLAN.md`

---

## ✦ 2. Features Implemented

### 2.1 Three-Column Now Playing Layout (`src/ui/views/now_playing.rs`)
Restructured the Now Playing view into an audiophile three-column layout directly aligned with `docs/reference/target-umbra.png`:
1. **Left Column (Artwork & Technical Badges)**:
   - Large rounded artwork card (`Rounding::same(14.0)`) with multi-layered specular violet ambient halo/shadow (`theme.accent_glow`).
   - High-contrast Track Title (`#f4f0ff`, 19.0px, bold), Artist (`#b2a8cf`, 14.5px), Album & Year (`#6d6488`, 12.5px).
   - Format Badges: Pill badges for Format (`FLAC`, `MP3`, etc.), Bit Depth & Sample Rate (e.g. `24-BIT 48 KHZ`, `16-BIT 44.1 KHZ`), Play Count (`▶ N plays`), and Lossless / Hi-Res glow tags.
2. **Center Column (Synced Lyrics)**:
   - Dynamic vertical scroll area (`ScrollArea::vertical()`) hosting synchronized `.lrc` lyrics.
   - **Auto-Scrolling**: Active lyric line automatically scrolls into view using `btn.scroll_to_me(Some(egui::Align::Center))`.
   - **Typography & Glow**: Current lyric line is highlighted in bright `#f4f0ff` (18.0px, bold) with translucent violet accent glow (`rgba(179, 136, 255, 30)`) and border (`theme.border_accent`); previous and upcoming lines smoothly dim in muted lavender (`#6d6488`, 14.5px).
   - **Click-to-Seek**: Clicking any lyric row immediately seeks playback to that exact millisecond timestamp (`player.engine.seek(Duration::from_millis(line.timestamp_ms))`).
3. **Right Column (Audio Analyzer Frosted Card)**:
   - Frosted panel frame (`theme.glass_frame(16.0)`).
   - Header with glowing `HI-RES LOSSLESS` or `LOSSLESS` badge in signature warm amber (`#ffb25c`).
   - **Technical Specs Sub-Card**: Codec/Format, Sample Rate (e.g. `48,000 Hz`), Bit Depth / Bitrate (e.g. `24-bit · 1,536 kbps`), and Channel configuration (e.g. `Stereo (2.0 ch)`).
   - **Real-Time Audio Metrics Sub-Card**: Instantaneous Peak level (`-1.2 dBFS` with clip alert in warm amber), Integrated Loudness (`-14.2 LUFS`), and Dynamic Range (`DR 12`).
   - **Live L/R Stereo Level Meters**: Dual animated horizontal meters tracking Left and Right channel signals with fast response, smooth decay, peak retention markers, and dB labels.
   - **20Hz - 20kHz Responsive Spectrum Visualizer**: Responsive frequency spectrum canvas with style toggles (`Bars`, `Mirror`, `Scope`) and frequency axis markers (`20Hz`, `100Hz`, `500Hz`, `1kHz`, `5kHz`, `10kHz`, `20kHz`).

### 2.2 History Navigation & Instant Search (`src/ui/titlebar.rs` & `src/ui/mod.rs`)
- **Browser-Grade History Navigation**:
  - Maintained `nav_history: Vec<NavTab>` history stack and pointer index in `UmbraApp`.
  - Implemented `navigate_to(tab)`, `can_go_back()`, `can_go_forward()`, `go_back()`, and `go_forward()`.
  - Titlebar navigation buttons (`‹` / `›`) dynamically enable/disable: disabled buttons render in muted translucency with default cursor, while active buttons render with borders, hover glow, pointer hand cursor, and instant navigation.
  - History branch truncation when navigating from prior history states.
- **Centered Search with Instant Multi-View Filtering**:
  - Pill search box with magnifying glass and `"Ctrl K"` shortcut hint.
  - Instant real-time filtering integrated across:
    - `Songs`: filters by title, artist, album, and genre.
    - `Albums`: filters album grid by album name and artist.
    - `Artists`: filters artist cards by artist name.
    - `Genres`: filters genres list by genre name.
  - Pressing `Enter` while typing in the global search bar automatically routes to `NavTab::Songs` to display full matched tracks.

### 2.3 Expanded Sidebar & Smart Playlists (`src/ui/sidebar.rs` & `src/library/db.rs`)
- **Expanded `NavTab` Hierarchy**:
  - `LIBRARY`: Home, Songs, Artists, Albums, Genres, Folders.
  - `SMART`: Recently Played, Most Played, Recently Added, Favorites, Never Played, Random Discovery.
  - `STREAM`: Spotify (Disabled badge), YouTube Music (Disabled badge).
  - `PLAYLISTS`: Section header with `+` quick-create button, user playlist rows with custom icons and track count pills.
  - `PREFERENCES`: Settings, About at bottom without overlapping statistics.
- **Smart Playlist Queries & UI**:
  - Added dedicated SQLite query methods in `Database`:
    - `get_smart_recently_played`: Retrieves playback history or tracks with `play_count > 0`.
    - `get_smart_most_played`: Retrieves tracks sorted by `play_count DESC`.
    - `get_smart_recently_added`: Retrieves tracks sorted by `added DESC, id DESC`.
    - `get_smart_favorites`: Retrieves favorited tracks (`favorite != 0`).
    - `get_smart_never_played`: Retrieves tracks with `play_count == 0`.
    - `get_smart_random_discovery`: Retrieves randomized library tracks via `ORDER BY RANDOM()`.
  - Unified `render_songs_view` supporting custom headers with emoji badges (`❤️`, `🔥`, `🕒`, `✨`, `⏳`, `🎲`, `🎶`), track stats, `▶ Play All`, and `🔀 Shuffle` quick actions.

### 2.4 Genres & About Views (`src/ui/views/genres.rs` & `src/ui/views/about.rs`)
- `genres.rs`: Grouping tracks by genre with contextual instrument icons (`🎸` Rock/Metal, `🎷` Jazz, `🎹` Classical, `🎧` Electronic, `🎤` Pop, `🎙️` Rap, `🌌` Ambient, `🎮` OST), interactive grid cards, and drill-down track views.
- `about.rs`: Frosted showcase card detailing Umbra v1.2.0, Rust architecture (`rodio`, `lofty`, `egui`, `rusqlite`), and feature capabilities.

---

## ✦ 3. Verification & Quality Gates

| Verification Step | Result | Details |
|---|---|---|
| `cargo check` | **PASSED** | 0 errors, 0 warnings |
| `cargo clippy` | **PASSED** | 0 warnings across all modules |
| `cargo test` | **PASSED** | 13/13 tests passing (including new visualizer metrics and navigation history tests) |
| `cargo build --release` | **PASSED** | Successfully compiled optimized release binary `umbra.exe` (11.35 MB) |
| Live Execution (`umbra.exe`) | **PASSED** | Launched cleanly, stable memory usage (~370 MB RAM), responsive event handling |

---

## ✦ 4. Verification Guide for QA Reviewer (Phase 1E)

1. **Titlebar & History Navigation**:
   - Launch `umbra.exe`. Notice `‹` is initially disabled.
   - Click **Songs**, then **Artists**, then **Albums**.
   - Click `‹` (Back): returns to Artists. `›` (Forward) is now enabled.
   - Click `›`: advances to Albums.
   - Click in search box (or press `Ctrl+K`), type a query: notice instant filtering across Songs, Albums, Artists, and Genres.
2. **Now Playing 3-Column View**:
   - Double click a track in Songs to start playback.
   - Click **Now Playing** in the sidebar (or bottom bar art).
   - Verify Column 1 (Left): Rounded art with ambient glow, Title, Artist, Album, Year, format badges (`FLAC`/format, bit depth/sample rate, play count).
   - Verify Column 2 (Center): Synced lyrics auto-scroll with bright `#f4f0ff` active line and faded non-active lines. Click on any lyric line to jump directly to that timestamp.
   - Verify Column 3 (Right): Audio Analyzer card displays format header badge (`HI-RES LOSSLESS` / `LOSSLESS` in warm amber), technical specs, live Peak dBFS / LUFS / Dynamic Range stats, animated L/R stereo meters, and responsive spectrum with frequency axis markers (`20Hz` to `20kHz`).
3. **Smart Playlists**:
   - Click **Favorites**, **Most Played**, **Recently Played**, **Recently Added**, **Never Played**, and **Random Discovery** in the sidebar.
   - Verify header card with icon, description, `▶ Play All`, and `🔀 Shuffle` buttons.
4. **Genres View**:
   - Click **Genres** in sidebar.
   - Verify genre cards with instrument icons and track counts. Click a card to view and play tracks.
5. **About View**:
   - Click **About** at bottom of sidebar. Verify system architecture and feature information.
