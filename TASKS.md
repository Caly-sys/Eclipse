# Task Backlog & Delegation Board

| Task ID | Phase | Title | Agent | Status | Dependencies |
|---|---|---|---|---|---|
| **TASK-00** | Phase 0 | Setup Protocol, Detect Framework, Establish Baseline | `orchestrator` | `DONE` | None |
| **TASK-1A** | Phase 1A | Audit & Fix Event Pipeline, Dragging, Controls & Shortcuts | `interaction` | `DONE` | TASK-00 |
| **TASK-1B** | Phase 1B | Bundle UI/Icon Fonts, CJK Fallback & Fix Layout Overlap | `fonts-rendering` | `DONE` | TASK-00 |
| **TASK-1C** | Phase 1C | Implement Purple Glass Design & Central Theme Tokens | `design` | `DONE` | TASK-1A, TASK-1B |
| **TASK-1D** | Phase 1D | Implement Now Playing 3-Column, Analyzer Card & Smart Lists | `feature` | `DONE` | TASK-1C |
| **TASK-1E** | Phase 1E | Full Regression Verification & Bug Sweep | `qa-reviewer` | `DONE` | TASK-1D |
| **TASK-2A** | Phase 2A | Implement Frameless Window Edge & Corner Resizing | `interaction` | `DONE` | TASK-1E |
| **TASK-2B** | Phase 2B | Enhance FLAC & Format Badge Visibility & High-Contrast Typography | `fonts-rendering` | `DONE` | TASK-1E |
| **TASK-2C** | Phase 2C | Implement Cover Art Extraction, Caching & UI Texture Rendering | `orchestrator` | `DONE` | TASK-1E |
| **TASK-2D** | Phase 2D | Phase 2 Regression Review & Release Binary Packaging | `qa-reviewer` | `DONE` | TASK-2A, TASK-2B, TASK-2C |
| **TASK-3A** | Phase 3A | Real-Time Audio DSP, 2048 Hann FFT, Live dBFS & Queue Fixes | `audio-dsp-debugger` | `DONE` | TASK-2D |
| **TASK-3B** | Phase 3B | Zero-Emoji Vector/SVG Icon System & Sidebar Layout Overhaul | `svg-sidebar-specialist` | `DONE` | TASK-2D |
| **TASK-3C** | Phase 3C | Responsive Grid Auto-Wrapping, Umbra Home Dashboard & Playerbar | `grid-playerbar-specialist` | `DONE` | TASK-2D |
| **TASK-3D** | Phase 3D | Zero-Emoji Audit, Full Regression Suite & Live Launch Testing | `integration-verifier` | `DONE` | TASK-3A, TASK-3B, TASK-3C |

---

## Task Details & Acceptance Criteria

### TASK-1A: Interaction Audit & Event Pipeline Fix
- **Agent**: `interaction`
- **Scope**:
  - Implement custom titlebar hit-testing: `ui.interact(rect, id, Sense::click_and_drag())` + `ctx.send_viewport_cmd(ViewportCommand::StartDrag)`.
  - Fix minimize, maximize/restore, and close buttons on the frameless window.
  - Implement interactive seek bar with direct click-to-seek and smooth dragging.
  - Fix volume slider and mute toggle.
  - Add independent sidebar scrolling and double-click row playback.
  - Add global keyboard shortcuts: `Space` (play/pause), `←`/`→` (seek ±5s), `↑`/`↓` (volume ±5%), `M` (mute), `S` (shuffle), `R` (repeat), `Ctrl+K` (focus search).
  - Create manual verification checklist in `docs/TESTING.md`.
- **Handoff**: `handoffs/TASK-1A.md`

---

### TASK-1B: Typography, CJK Fallback & Layout Fitting
- **Agent**: `fonts-rendering`
- **Scope**:
  - Configure `egui::FontDefinitions` with Japanese, Chinese, and Korean fallback glyphs (Segoe UI, Yu Gothic, Meiryo, MS Gothic) so Japanese track names like `君はロックを聴かない` and lyrics render with zero empty boxes / tofu.
  - Bundle clear symbol icon glyphs for media playback controls.
  - Fix sidebar layout overlap so track/playlist statistics never obscure Settings/About.
  - Remove fixed height subtractions so viewport content dynamically fills available window height.
- **Handoff**: `handoffs/TASK-1B.md`

---

### TASK-1C: Purple Glass Design & Central Theme Module
- **Agent**: `design`
- **Scope**:
  - Refactor `src/ui/theme.rs` to serve as the single source of truth for color, spacing, radius, and font tokens.
  - Implement the purple glass aesthetic matching `docs/reference/target-umbra.png`:
    - Canvas `#0b0914`, frosted panels `#141022`, translucent borders `rgba(180, 140, 255, 0.15)`.
    - Neon violet `#b388ff` primary accent, warm amber `#ffb25c` secondary badge accent.
  - Ensure all views and widgets consume tokens from `Theme` rather than hardcoding colors.
- **Handoff**: `handoffs/TASK-1C.md`

---

### TASK-1D: Target Screens & Missing Features
- **Agent**: `feature`
- **Scope**:
  - Now Playing 3-column view:
    - Left: Large rounded artwork (`260x260px`) with ambient drop shadow, title, artist, year, format badges (`FLAC`, `24-BIT 48 KHZ`, plays).
    - Center: Synced lyrics with bright active line and faded non-active lines; click-to-seek.
    - Right: Audio Analyzer card with Peak, LUFS, Dynamic Range, L/R meters, and live 20Hz-20kHz spectrum.
  - Top bar Back/Forward navigation buttons and centered search box with `Ctrl K` shortcut hint.
  - Smart playlist views: Recently Played, Most Played, Recently Added, Favorites, Never Played, Random Discovery.
- **Handoff**: `handoffs/TASK-1D.md`

---

### TASK-1E: End-to-End QA Review & Bug Verification
- **Agent**: `qa-reviewer`
- **Scope**:
  - Run full verification suite (`cargo check`, `cargo clippy`, `cargo test`).
  - Run `umbra.exe` live, execute all manual tests from `docs/TESTING.md`.
  - Test audio playback across FLAC/MP3 files in `C:\music\Muse`.
  - Log any new or remaining defects in `BUGS.md`.
- **Handoff**: `handoffs/TASK-1E.md`

---

### TASK-2A: Frameless Window Edge & Corner Resizing
- **Agent**: `interaction`
- **Scope**:
  - Implement window perimeter border resize detection (`src/ui/resize.rs`) with 7px margin along all 4 edges and corners.
  - Wire directional CursorIcon (`ResizeNorth`, `ResizeSouth`, `ResizeWest`, `ResizeEast`, `ResizeNorthWest`, `ResizeNorthEast`, `ResizeSouthWest`, `ResizeSouthEast`).
  - Wire native OS resizing via `ViewportCommand::BeginResize(resize_direction)` on primary mouse press.
  - Add visible glass resize grip handle (`///` lines) in bottom-right corner of docked playerbar.
  - Ensure titlebar dragging and child button clicks are not blocked.
- **Handoff**: `handoffs/TASK-2A.md`

---

### TASK-2B: Enhance FLAC & Format Badge Visibility & High-Contrast Typography
- **Agent**: `fonts-rendering`
- **Scope**:
  - Implement `format_badge` in `src/ui/components.rs` with bold warm amber (`#ffb25c`) fill/border for lossless (FLAC/WAV/ALAC) and bright lilac (`#ebe4ff`) for lossy (MP3/AAC/OGG).
  - Align table headers and song rows in `src/ui/views/songs.rs` with dedicated 60px centered format columns.
  - Upgrade duration timestamps to high-contrast `theme.text_secondary`.
  - Add `format_badge` to playerbar, now playing, home favorites, and playlist views.
- **Handoff**: `handoffs/TASK-2B.md`

---

### TASK-2C: Cover Art Extraction, Caching & UI Texture Rendering
- **Agent**: `orchestrator`
- **Scope**:
  - Implement embedded tag cover art extraction (`lofty`) and local folder fallback (`cover.jpg`/`folder.jpg`) in `src/library/scanner.rs`.
  - Implement disk cache (`%APPDATA%\Umbra\art_cache\<hash>.bin`).
  - Implement `AlbumArtManager` with GPU texture caching and 512x512 downsampling in `src/ui/art.rs`.
  - Wire real album cover rendering into Now Playing view, Playerbar mini thumbnail, Albums grid cards & header, and Home view.
- **Handoff**: `handoffs/TASK-2C.md`

---

### TASK-2D: Phase 2 Regression Review & Release Binary Packaging
- **Agent**: `qa-reviewer`
- **Scope**:
  - Run full verification suite (`cargo check`, `cargo clippy`, `cargo test`).
  - Run `umbra.exe` live, test window resizing from all 4 borders, corners, and grip handle.
  - Test FLAC format badge contrast and alignment.
  - Test album cover art display in Now Playing, bottom playerbar, and Albums grid.
  - Verify release build packaging.
- **Handoff**: `handoffs/TASK-2D.md`

---

### TASK-3A: Real-Time Audio DSP, 2048 Hann FFT, Live dBFS & Queue Fixes
- **Agent**: `audio-dsp-debugger`
- **Scope**:
  - Implement real PCM audio tap on rodio output sink / decoder stream using ring buffer.
  - Implement 2048 Hann-windowed FFT via `rustfft`, map to 20Hz-20kHz log bins with dBFS temporal decay.
  - Provide true time-domain PCM for Scope, true L/R RMS and Peak dBFS for Stereo Meters.
  - Fix Next/Prev skipping, ensure queue populates from any view, respect Shuffle/Repeat, advance track on EOF.
  - STRICT: NO fake waves, ZERO emojis.
- **Handoff**: `handoffs/TASK-3A.md`

---

### TASK-3B: Zero-Emoji Vector/SVG Icon System & Sidebar Layout Overhaul
- **Agent**: `svg-sidebar-specialist`
- **Scope**:
  - Create `src/ui/icons.rs` with clean Lucide-style vector/SVG icons rendered via egui painter/shapes.
  - Purge 100% of Unicode emojis and text symbols across the codebase.
  - Fix sidebar width to fixed 240px, fix scroll area padding (12, 10).
  - Fix YouTube MusicDisabled collision in STREAM section with compact pill badge.
  - Full-width footer library statistics card with vector icon.
  - High-contrast active navigation pill with purple glowing border/background, light lavender text, and 3px left accent indicator bar.
  - STRICT: ZERO emojis.
- **Handoff**: `handoffs/TASK-3B.md`

---

### TASK-3C: Responsive Grid Auto-Wrapping, Umbra Home Dashboard & Playerbar
- **Agent**: `grid-playerbar-specialist`
- **Scope**:
  - Fix Albums & Artists card squishing with dynamic auto-wrapping responsive grid.
  - Rebuild Umbra Home dashboard: Hero banner + Quick Actions + 4-card stats grid, Recently Played shelf, Recently Added & Most Played sections, Favorites grid.
  - Restructure Player Bar into 3-column layout: Left track snippet + format badge + heart, Center transport + seekbar, Right mini spectrum + volume + toggles.
  - Fix overlapping frequency axis labels in Audio Analyzer (`20Hz...20kHz`).
  - STRICT: ZERO emojis.
- **Handoff**: `handoffs/TASK-3C.md`

---

### TASK-3D: Zero-Emoji Audit, Full Regression Suite & Live Launch Testing
- **Agent**: `integration-verifier`
- **Scope**:
  - Audit codebase for 0 emojis.
  - `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all`, `cargo build --release`.
  - Test live `umbra.exe` execution, responsiveness, and sync directories.
- **Handoff**: `handoffs/TASK-3D.md`




