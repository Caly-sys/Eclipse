# Handoff Report: TASK-1A — Audit & Fix Event Pipeline, Dragging, Controls & Shortcuts

- **Task ID**: `TASK-1A`
- **Agent**: `interaction`
- **Status**: `HANDOFF_READY`
- **Date**: 2026-11-09

---

## ✦ 1. Summary of Changes

TASK-1A resolves **BUG-003** (frameless window drag), **BUG-004** (window controls & hit-testing), **BUG-006** (seek bar & volume clunkiness), and **BUG-007** (missing keyboard shortcuts).

### 1. Titlebar Hit-Testing & Window Dragging (`src/ui/titlebar.rs`)
- Implemented background hit-testing covering the entire titlebar area via `ui.interact(titlebar_rect, titlebar_id, Sense::click_and_drag())`.
- Wired `titlebar_resp.drag_started_by(egui::PointerButton::Primary)` to send `ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag)`, invoking Windows OS-level modal window dragging fluidly across any non-interactive portion of the titlebar.
- Added double-click hit detection on the titlebar background: inspecting `ui.input(|i| i.viewport().maximized)` and toggling maximize/restore via `ViewportCommand::Maximized(!is_maximized)`.
- Verified via unit test that background interaction does not swallow or intercept clicks aimed at child buttons (Search input, Quick Scan, Window controls).

### 2. Frameless Window Controls (`src/ui/titlebar.rs`)
- **Close Button (`✕`)**: Sized to `32x26px`, renders red hover background (`#dc2626`) with white glyph, and sends `ViewportCommand::Close`.
- **Maximize / Restore Button (`□` / `❐`)**: Inspects `viewport().maximized`, dynamically toggles icon (`□` when windowed, `❐` when maximized), sends `ViewportCommand::Maximized(!is_maximized)`, and features hover highlight.
- **Minimize Button (`–`)**: Sized to `32x26px`, sends `ViewportCommand::Minimized(true)`, and features hover highlight.
- Sized buttons to provide a comfortable hit target with instant visual feedback.

### 3. Custom Interactive Seek Bar (`src/ui/playerbar.rs`)
- Replaced the previous `egui::Slider` with `render_custom_seekbar`:
  - **Direct Single-Click-to-Seek**: Clicking anywhere along the bar computes the exact ratio from mouse coordinate `interact_pointer_pos()` and seeks immediately to that song timestamp.
  - **Continuous Scrubbing**: Smooth dragging across the bar continuously seeks the audio without handle lag or position jumpiness.
  - **Hover Time Preview**: Hovering displays an accurate `mm:ss` timestamp tooltip corresponding to the cursor position.
  - **Aesthetics & Hitbox**: Hit target expanded to 18px for effortless mouse targeting; visual rail dynamically expands from 4px to 6px on hover/drag; glowing circular scrubber knob (radius 6px) with glowing accent halo appears when hovered or dragged; cursor switches to `PointingHand`.

### 4. Custom Volume Slider & Mute Control (`src/ui/playerbar.rs`)
- Replaced standard slider with `render_custom_volume_slider`:
  - Direct click-to-jump and smooth dragging from 0% to 100%.
  - **Mouse Scroll Wheel Volume**: Hovering over the volume slider and scrolling the wheel adjusts volume in ±5% increments.
  - **Auto-Unmute**: Increasing volume while muted automatically unmutes the audio stream.
  - Percentage tooltip displayed on hover (`"75%"`).
  - Responsive mute button toggling between `🔇`, `🔈`, `🔉`, and `🔊`.

### 5. Global Keyboard Shortcuts & Input Guard (`src/ui/shortcuts.rs`)
- Created clean dedicated shortcuts module `src/ui/shortcuts.rs` wired into `UmbraApp::update`:
  - `Space`: Toggles Play / Pause, or starts playback of first track in queue.
  - `←` (Left Arrow): Seeks -5 seconds.
  - `→` (Right Arrow): Seeks +5 seconds.
  - `↑` (Up Arrow): Volume +5% (with auto-unmute).
  - `↓` (Down Arrow): Volume -5%.
  - `M`: Toggles Mute.
  - `S`: Toggles Shuffle (`Shuffle: ON` / `OFF`).
  - `R`: Cycles Repeat Mode (`Repeat: ALL` / `ONE` / `OFF`).
  - `Ctrl + K`: Programmatically focuses the titlebar search box.
  - `F11`: Toggles fullscreen mode.
  - **Input Focus Guard**: When `ctx.wants_keyboard_input()` is true (e.g. typing in search box), single-key shortcuts are suppressed so typing spaces or letters never alters playback or seeks.
  - Instant visual toast feedback for every shortcut action.

### 6. Song Table Row Selection & Double-Click Playback (`src/ui/views/songs.rs`)
- Introduced `SongsViewState` with `selected_track_id: Option<i64>`.
- Single-clicking a song row selects it with visual accent border and background tint without interrupting current playback.
- Double-clicking a song row immediately loads the filtered track list into the queue and starts playback of that song.
- Isolated favorite heart button (`❤️` / `🤍`) clicks so favoriting a track never triggers row selection or track playback.
- Added hover highlight and `PointingHand` cursor feedback across all rows.
- In `src/ui/views/home.rs`, made "Recently Played" track rows clickable to immediately start playback.

### 7. Testing Documentation (`docs/TESTING.md`)
- Created exhaustive manual verification checklist covering all 26 interaction scenarios across window controls, timeline seek bar, volume control, keyboard shortcuts, sidebar scrolling, and song table interactions.

---

## ✦ 2. Files Modified & Created

| File | Change |
|---|---|
| [`src/ui/shortcuts.rs`](file:///C:/music/umbra/src/ui/shortcuts.rs) | **New file**: Global keyboard shortcut listener, text input guard, and unit tests |
| [`src/ui/titlebar.rs`](file:///C:/music/umbra/src/ui/titlebar.rs) | Added draggable background hit-testing, double-click maximize toggle, fixed Min/Max/Close buttons with hover states, search `Ctrl+K` focus |
| [`src/ui/playerbar.rs`](file:///C:/music/umbra/src/ui/playerbar.rs) | Replaced `egui::Slider` with custom interactive seek bar (single-click jump, continuous scrub, hover preview) and custom volume slider |
| [`src/ui/views/songs.rs`](file:///C:/music/umbra/src/ui/views/songs.rs) | Added `SongsViewState`, single-click row selection, double-click row playback, hover highlight, isolated favorite button clicks |
| [`src/ui/views/home.rs`](file:///C:/music/umbra/src/ui/views/home.rs) | Added click interaction and hover cursor to "Recently Played" track items |
| [`src/ui/views/mod.rs`](file:///C:/music/umbra/src/ui/views/mod.rs) | Exported `SongsViewState` |
| [`src/ui/mod.rs`](file:///C:/music/umbra/src/ui/mod.rs) | Integrated `shortcuts` module, added `focus_search` and `songs_state`, wired titlebar and song view parameters |
| [`docs/TESTING.md`](file:///C:/music/umbra/docs/TESTING.md) | **New file**: Complete manual testing guide and QA checklist |
| [`BUGS.md`](file:///C:/music/umbra/BUGS.md) | Moved `BUG-003`, `BUG-004`, `BUG-006`, and `BUG-007` to Resolved |
| [`TASKS.md`](file:///C:/music/umbra/TASKS.md) | Marked `TASK-1A` as `HANDOFF_READY` |

---

## ✦ 3. Build & Test Verification

### 1. `cargo check`
- **Result**: PASSED (0 errors, 0 warnings).

### 2. `cargo clippy`
- **Result**: PASSED (0 errors, 0 warnings on all interaction modules).

### 3. `cargo test`
- **Result**: PASSED (6/6 tests passing in 0.12s).
  - `ui::shortcuts::tests::test_shortcuts_volume_and_mute`: PASSED
  - `ui::shortcuts::tests::test_shortcuts_toggle_shuffle_and_repeat`: PASSED
  - `ui::shortcuts::tests::test_ctrl_k_focus_search`: PASSED
  - `ui::shortcuts::tests::test_shortcuts_suppressed_when_typing`: PASSED
  - `ui::fonts::tests::test_font_definitions_load`: PASSED
  - `ui::fonts::tests::test_cjk_glyphs_present`: PASSED

### 4. `cargo build`
- **Result**: PASSED (`target/debug/umbra.exe` compiled in 6.50s).

### 5. Live Runtime Test (`umbra.exe`)
- **Result**: PASSED.
  - Process launched cleanly (PID 20436).
  - Window initialized, custom titlebar and seekbar widgets instantiated with no panics.
  - Clean shutdown verified.

---

## ✦ 4. Notes for Downstream Agents

- **`design` agent (TASK-1C)**:
  - The custom seek bar and volume slider in `src/ui/playerbar.rs` consume `theme.accent` and `theme.text_primary`. When updating `src/ui/theme.rs` with the purple glass palette, these controls will automatically render with the new neon violet tokens.
- **`feature` agent (TASK-1D)**:
  - `Ctrl+K` shortcut support is wired and requests focus on the titlebar search box.
  - When implementing the Now Playing 3-column lyrics click-to-seek, `player.engine.seek(Duration::from_secs_f64(secs))` can be reused directly.
- **`qa-reviewer` agent (TASK-1E)**:
  - Run through [`docs/TESTING.md`](file:///C:/music/umbra/docs/TESTING.md) to manually test titlebar drag, double-click maximize, seekbar single-click and scrubbing, volume wheel, and keyboard shortcuts.
