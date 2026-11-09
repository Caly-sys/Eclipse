# Handoff Report: TASK-1B — Typography, CJK Fallback & Layout Fitting

- **Task ID**: `TASK-1B`
- **Agent**: `fonts-rendering`
- **Status**: `HANDOFF_READY`
- **Date**: 2026-11-09

---

## ✦ 1. Summary of Changes

TASK-1B resolves **BUG-002** (missing CJK tofu characters) and **BUG-005** (sidebar overlap and hardcoded viewport layout subtractions).

### 1. Typography & CJK Fallback (`src/ui/fonts.rs`)
- Implemented `setup_custom_fonts()` in `src/ui/fonts.rs` to configure `egui::FontDefinitions`.
- Loaded Windows high-quality CJK fonts in priority order:
  1. `segoeui.ttf` (Segoe UI — primary crisp Western typography)
  2. `YuGothM.ttc` / `YuGothR.ttc` (Yu Gothic — modern Windows Japanese font)
  3. `msgothic.ttc` (MS Gothic — Japanese fallback)
  4. `msyh.ttc` (Microsoft YaHei — Simplified Chinese)
  5. `malgun.ttf` (Malgun Gothic — Korean & CJK ideographs)
  6. `seguisym.ttf` (Segoe UI Symbol — crisp media playback symbols)
  7. `seguiemj.ttf` (Segoe UI Emoji — clean emoji glyphs)
- Registered fallback order for both `egui::FontFamily::Proportional` and `egui::FontFamily::Monospace`.
- Defined standard media control constants in `symbols` module (`PLAY`, `PAUSE`, `PREV`, `NEXT`, `SHUFFLE`, `REPEAT`, `HEART_FILLED`, `HEART_OUTLINE`, `VOLUME_MUTE`, `VOLUME_LOW`, `VOLUME_MED`, `VOLUME_HIGH`, `SEARCH`, `QUEUE`, `EXPAND`, `SPARKLE`, `REFRESH`).
- Japanese track names in `C:\music\Muse` (e.g. `君はロックを聴かない`, `ただ君に晴れ`, `晩餐歌`, `革命道中 - On The Way`, `ラグトレイン`, `感電`, `星座になれたら`, `リテラチュア`, `カワキヲアメク`, `だから僕は音楽を辞めた`) and lyrics render completely without empty box characters (`[]`).

### 2. Sidebar Navigation & Layout Overlap Fix (`src/ui/sidebar.rs`)
- Placed all sidebar navigation buttons inside an independent `ScrollArea::vertical().auto_shrink([false, false])`.
- Removed `Layout::bottom_up` which was causing the bottom library count card to collide with and obscure Settings / navigation buttons at small window heights (e.g. 600px).
- Library track and playlist statistics now flow linearly in the vertical area without ever clipping or overlapping buttons.

### 3. Dynamic Window Height Fitting (`src/ui/mod.rs`)
- Removed hardcoded height subtractions (`available_height() - 75.0`).
- Dynamically calculate `workspace_height` from the measured height of the docked bottom playerbar (`playerbar_id` via `ctx.data` and `response.rect.height()`).
- Workspace panels (Sidebar, Content Area, Play Queue drawer) dynamically fill the exact available window height.

### 4. Text Truncation & Clipping Fixes (`src/ui/views/songs.rs`, `src/ui/views/albums.rs`)
- Added `.truncate()` to song title, artist, and album labels in `songs.rs` to prevent long track titles from breaking table column widths.
- Added character-aware truncation with ellipsis to album titles and artist names in `albums.rs` to prevent text from overflowing across neighboring grid cards.

---

## ✦ 2. Files Modified & Created

| File | Change |
|---|---|
| [`src/ui/fonts.rs`](file:///C:/music/umbra/src/ui/fonts.rs) | **New file**: FontDefinitions loader with CJK fallback priority, symbols module, and unit tests |
| [`src/ui/mod.rs`](file:///C:/music/umbra/src/ui/mod.rs) | Registered `fonts` module, initialized `fonts::setup_custom_fonts()`, and replaced `- 75.0` with dynamic height calculation |
| [`src/ui/sidebar.rs`](file:///C:/music/umbra/src/ui/sidebar.rs) | Wrapped sidebar navigation in `ScrollArea::vertical()` and eliminated `Layout::bottom_up` collisions |
| [`src/ui/playerbar.rs`](file:///C:/music/umbra/src/ui/playerbar.rs) | Return `egui::Response` to allow parent to measure playerbar height dynamically; resolved unused variable |
| [`src/ui/views/songs.rs`](file:///C:/music/umbra/src/ui/views/songs.rs) | Added `.truncate()` to title, artist, album columns to prevent layout stretching |
| [`src/ui/views/albums.rs`](file:///C:/music/umbra/src/ui/views/albums.rs) | Added card boundary truncation for album title and artist text |
| [`BUGS.md`](file:///C:/music/umbra/BUGS.md) | Moved `BUG-002` and `BUG-005` to Resolved |
| [`TASKS.md`](file:///C:/music/umbra/TASKS.md) | Updated `TASK-1B` to `HANDOFF_READY` |

---

## ✦ 3. Build & Test Verification

### 1. `cargo check`
- **Result**: PASSED (0 errors, 0 warnings).

### 2. `cargo test`
- **Result**: PASSED (2/2 tests passed in 0.12s).
  - `ui::fonts::tests::test_font_definitions_load`: PASSED
  - `ui::fonts::tests::test_cjk_glyphs_present`: PASSED (verified glyph resolution for 10 sample Japanese track titles)

### 3. `cargo clippy`
- **Result**: PASSED (0 errors, 6 pre-existing non-blocking style warnings).

### 4. Live Runtime Test (`umbra.exe`)
- **Result**: PASSED.
  - Process launched cleanly with PID 9396.
  - UI initialized, window rendered, font definitions loaded without panic.
  - Clean exit verified.

---

## ✦ 4. Notes for Downstream Agents

- **`design` agent (TASK-1C)**:
  - Font definitions are ready and initialized in `src/ui/fonts.rs`.
  - When updating `src/ui/theme.rs` and components, all CJK and symbol glyphs will render smoothly with any font sizes or theme colors.
- **`qa-reviewer` agent (TASK-1E)**:
  - Check Japanese songs in `C:\music\Muse` (e.g. Aimyon, Yorushika, Kenshi Yonezu) in the Songs view and Now Playing lyrics.
  - Resize the window to the minimum size (`950x600`) and verify that the sidebar scrolls smoothly without the bottom library statistics overlapping Settings.
