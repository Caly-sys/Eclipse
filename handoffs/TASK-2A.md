# Handoff Report: TASK-2A — Frameless Window Edge & Corner Resizing

- **Task ID**: `TASK-2A`
- **Agent**: `interaction`
- **Status**: `HANDOFF_READY`
- **Date**: 2026-11-09

---

## ✦ 1. Summary of Changes

TASK-2A resolves the user feedback regarding frameless window resizing: *"reisizing it is kinda hard rn other tho its fine"*. Because Umbra is configured as a frameless dark glass window (`with_decorations(false)`), native window borders are replaced with custom border edge & corner hit-testing and a visual corner resize grip handle.

### 1. Perimeter Edge & Corner Hit-Testing (`src/ui/resize.rs`)
- Implemented `detect_resize_direction(screen_rect, pos, margin)` and `handle_window_resize(ctx)` in `src/ui/resize.rs`:
  - Uses a comfortable `7.0px` margin along all 4 edges and corners of the window screen rect.
  - Detects all 8 resize directions with corner priority over edges:
    - **Top-Left Corner**: `ResizeDirection::NorthWest` (`CursorIcon::ResizeNorthWest`)
    - **Top-Right Corner**: `ResizeDirection::NorthEast` (`CursorIcon::ResizeNorthEast`)
    - **Bottom-Left Corner**: `ResizeDirection::SouthWest` (`CursorIcon::ResizeSouthWest`)
    - **Bottom-Right Corner**: `ResizeDirection::SouthEast` (`CursorIcon::ResizeSouthEast`)
    - **Top Edge**: `ResizeDirection::North` (`CursorIcon::ResizeNorth`)
    - **Bottom Edge**: `ResizeDirection::South` (`CursorIcon::ResizeSouth`)
    - **Left Edge**: `ResizeDirection::West` (`CursorIcon::ResizeWest`)
    - **Right Edge**: `ResizeDirection::East` (`CursorIcon::ResizeEast`)
  - Automatically updates the viewport cursor icon to the appropriate directional resize cursor.
  - On primary mouse button press (`i.pointer.primary_pressed()`), dispatches `ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction))`.
  - On Windows, `BeginResize` invokes the OS window manager's native modal sizing loop (`WM_NCLBUTTONDOWN` with `HT*` hit codes), ensuring smooth and stutter-free window resizing across all monitor configurations.
  - Automatically suppresses resizing when the window is maximized (`viewport().maximized == Some(true)`).

### 2. Titlebar Hit-Testing Priority Guard (`src/ui/titlebar.rs`)
- Added `pointer_in_resize` perimeter guard in `src/ui/titlebar.rs`:
  - When the cursor is within the top 7px or top corners of the titlebar, `StartDrag` and double-click maximize are suppressed.
  - This ensures that attempting to resize the window from the top border or top corners immediately engages `BeginResize(North)` / `BeginResize(NorthWest)` / `BeginResize(NorthEast)` rather than dragging the window.

### 3. Visual Glass Resize Grip Handle in Playerbar (`src/ui/playerbar.rs`)
- Implemented `render_resize_grip(ui, theme)` in the bottom-right corner of the docked playerbar:
  - Draws 3 subtle glass diagonal lines (`///`) in the bottom-right corner.
  - In idle state, lines render with translucent lavender `rgba(180, 140, 255, 0.3)`.
  - On hover or drag, lines glow in primary neon violet `theme.accent` (`#b388ff`).
  - Cursor switches to `CursorIcon::ResizeSouthEast`.
  - Tooltip displays `"Drag to resize window"`.
  - Clicking/dragging initiates native `BeginResize(ResizeDirection::SouthEast)`.
  - Gives users a visible visual affordance to grab and resize the window without pixel-hunting edges.

### 4. Non-Interference with Child Widgets
- Edge hit-testing operates strictly in the outer 7px perimeter.
- All application widgets (Titlebar buttons, Close/Min/Max buttons, Search bar, Sidebar tabs, Playerbar controls, and Content area) reside within `CentralPanel` with `10.0px` margin, completely outside the 7px border zone. No child button clicks are blocked.

---

## ✦ 2. Files Modified & Created

| File | Change |
|---|---|
| [`src/ui/resize.rs`](file:///C:/music/umbra/src/ui/resize.rs) | **New file**: Window edge & corner detection module, cursor handling, viewport resize dispatch, and 6 unit tests |
| [`src/ui/mod.rs`](file:///C:/music/umbra/src/ui/mod.rs) | Registered `pub mod resize;` and called `resize::handle_window_resize(ctx)` in `UmbraApp::update` |
| [`src/ui/titlebar.rs`](file:///C:/music/umbra/src/ui/titlebar.rs) | Added `pointer_in_resize` guard to prevent `StartDrag` from overriding top edge/corner resizing |
| [`src/ui/playerbar.rs`](file:///C:/music/umbra/src/ui/playerbar.rs) | Added `render_resize_grip` with diagonal lines (`///`) and `ResizeSouthEast` native resize handling |
| [`docs/TESTING.md`](file:///C:/music/umbra/docs/TESTING.md) | Added test cases `WB-06` (border edges), `WB-07` (corners), and `WB-08` (bottom-right grip handle) |
| [`TASKS.md`](file:///C:/music/umbra/TASKS.md) | Marked `TASK-2A` as `HANDOFF_READY` with scope and criteria |

---

## ✦ 3. Build & Test Verification

### 1. `cargo check`
- **Result**: PASSED (0 errors, 0 warnings).

### 2. `cargo clippy`
- **Result**: PASSED (0 errors, 0 warnings).

### 3. `cargo test`
- **Result**: PASSED (23/23 tests passing in 0.11s).
  - `ui::resize::tests::test_resize_corner_detection`: PASSED (all 4 corners verified)
  - `ui::resize::tests::test_resize_edge_detection`: PASSED (all 4 edges verified)
  - `ui::resize::tests::test_interior_points_return_none`: PASSED
  - `ui::resize::tests::test_outside_window_returns_none`: PASSED
  - `ui::resize::tests::test_handle_window_resize_sets_cursor_and_command`: PASSED (verifies `CursorIcon::ResizeWest` and `ViewportCommand::BeginResize(West)`)
  - `ui::resize::tests::test_handle_window_resize_disabled_when_maximized`: PASSED (verifies resizing disabled when maximized)

### 4. `cargo build`
- **Result**: PASSED (`target/debug/umbra.exe` compiled in 5.68s).

### 5. Live Runtime Test (`umbra.exe`)
- **Result**: PASSED.
  - Process launched cleanly (PID 4132).
  - All UI elements and new resize grip handle rendered cleanly without panics.
  - Clean exit confirmed.

---

## ✦ 4. Notes for Downstream Agents & QA

- **`qa-reviewer`**:
  - Test all 4 edges (Left, Right, Top, Bottom) and 4 corners by hovering slowly over the window borders — verify that the cursor changes to the matching directional double-headed resize arrow and dragging resizes smoothly.
  - Test the bottom-right corner grip handle in the playerbar: hover over the diagonal `///` lines, verify `ResizeSouthEast` cursor and tooltip, then click and drag to resize both width and height.
  - Maximize the window (`□` or double-click titlebar): verify that border resizing and the grip handle are cleanly disabled while maximized.
