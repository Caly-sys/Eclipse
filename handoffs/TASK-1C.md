# Handoff: TASK-1C - Implement Purple Glass Design & Central Theme Tokens

## ✦ 1. Task Summary
- **Task ID**: `TASK-1C`
- **Agent**: `design`
- **Status**: `HANDOFF_READY`
- **Target Aesthetic**: Purple Glass theme derived directly from `docs/reference/target-umbra.png` and `DESIGN.md`.

---

## ✦ 2. Changes Implemented

### 2.1 Central Design Tokens Module (`src/ui/theme.rs`)
- Refactored `Theme` into a centralized, single-source-of-truth design tokens module.
- Exact token definitions matching `DESIGN.md`:
  - `bg`: Deep obsidian/charcoal purple `#0b0914` (`Color32::from_rgb(11, 9, 20)`)
  - `surface` (frosted glass panels): `rgba(20, 16, 34, 0.75)` (`Color32::from_rgba_premultiplied(20, 16, 34, 190)`)
  - `card` (sub-cards & containers): `rgba(28, 22, 48, 0.65)` (`Color32::from_rgba_premultiplied(28, 22, 48, 165)`)
  - `surface_hover`: `Color32::from_rgba_premultiplied(38, 30, 65, 200)`
  - `border`: Hairline translucent lavender `rgba(180, 140, 255, 0.14)` (`Color32::from_rgba_premultiplied(180, 140, 255, 36)`)
  - `border_accent`: `rgba(180, 140, 255, 0.35)` (`Color32::from_rgba_premultiplied(180, 140, 255, 90)`)
  - `accent`: Radiant neon violet `#b388ff` (`Color32::from_rgb(179, 136, 255)`)
  - `accent_glow`: Specular halo `Color32::from_rgba_premultiplied(179, 136, 255, 90)`
  - `accent_warm`: Warm amber `#ffb25c` (`Color32::from_rgb(255, 178, 92)`) for lossless badges, Hi-Res tags, and peak visualizer markers
  - `text_primary`: High-contrast `#f4f0ff` (`Color32::from_rgb(244, 240, 255)`)
  - `text_secondary`: Soft lilac-grey `#b2a8cf` (`Color32::from_rgb(178, 168, 207)`)
  - `text_muted`: Muted lavender `#6d6488` (`Color32::from_rgb(109, 100, 136)`)
- Added ergonomic standardized frame constructors:
  - `glass_frame(&self, rounding: f32) -> Frame`: Structural frosted glass panels
  - `card_frame(&self, rounding: f32) -> Frame`: Sub-cards and content containers
  - `badge_frame(&self, bg: Color32, border: Color32) -> Frame`: Standard pill badges
  - `panel_frame(&self) -> Frame`: Deep obsidian canvas background
- Standardized theme presets: `Eclipse` (default), `Obsidian`, `Glacier`, `Nebula`, `Verdant`, `Ember`.

### 2.2 Configuration Migration (`src/config.rs`)
- Updated default `accent_color` to `[179, 136, 255]` (`#b388ff`).
- In `AppConfig::load()`, automatically migrated configs with the previous `[255, 178, 92]` amber default to `[179, 136, 255]` so existing user setups instantly receive the signature purple glass visual aesthetic.

### 2.3 Component & View Restyling Matching `docs/reference/target-umbra.png`
1. **Titlebar (`src/ui/titlebar.rs`)**:
   - Glowing circular brand badge (`✦`) with ambient purple halo.
   - Bold uppercase `"UMBRA"` wordmark in high-contrast text.
   - History navigation buttons (`‹`, `›`) with glass borders.
   - Pill-shaped frosted search bar with `"Ctrl K"` pill shortcut badge and magnifying glass.
   - Window controls (`–`, `□`, `✕`) with frosted hover feedback and red close state.
2. **Sidebar (`src/ui/sidebar.rs`)**:
   - Frosted panel background using `theme.glass_frame(12.0)`.
   - Small uppercase section headers in muted purple typography (`LIBRARY`, `PLAYLISTS`, `PREFERENCES`).
   - Active tab highlighting: neon violet glow (`theme.accent_glow`), active left indicator pill (3.5px neon violet vertical bar), and glowing text.
   - Hover states with translucent lavender tint.
   - Library statistics card styled with `theme.card_frame(8.0)`.
3. **Playerbar (`src/ui/playerbar.rs`)**:
   - Frosted glass docked bar using `theme.glass_frame(12.0)`.
   - Mini track artwork with border and format pill badge in signature warm amber (`#ffb25c`).
   - Glowing circular play/pause button (`36x36px`) with outer ambient halo (`theme.accent_glow`), glowing neon violet body, white border, and dark charcoal icon.
   - Seek bar and volume slider styled with translucent rail tracks (`rgba(180, 140, 255, 30)`), neon violet fill rails, and glowing scrubber knobs.
4. **Components (`src/ui/components.rs`)**:
   - `pill_badge` now applies translucent hairline borders matching text color.
   - `render_spectrum` renders peak dots in warm amber (`#ffb25c`) against neon violet spectrum bars for the signature visual contrast.
5. **Views (`src/ui/views/*.rs`)**:
   - `home.rs`: Frosted hero banner, interactive favorite cards with pointer hand cursors, and warm amber format badges on recently played tracks.
   - `songs.rs`: Glass header row with hairline border, current playing track row highlighted with `theme.accent_glow` and `theme.border_accent`, alternating rows, and warm amber lossless badges.
   - `now_playing.rs`: Two-column layout with 16px rounded frosted glass artwork card, live spectrum analyzer card, synced lyrics with bright `#f4f0ff` current line and faded `#6d6488` past/future lines.
   - `albums.rs`, `artists.rs`, `folders.rs`, `playlists.rs`, `settings.rs`: All card frames, headers, and buttons updated to central tokens.

---

## ✦ 3. Verification & Quality Gates

| Verification Step | Result | Notes |
|---|---|---|
| `cargo check` | **PASSED** | 0 errors, 0 warnings |
| `cargo clippy` | **PASSED** | Clean on all UI and design modules |
| `cargo test` | **PASSED** | 9/9 tests passed (including 3 new Theme unit tests) |
| Live Execution (`umbra.exe`) | **PASSED** | Executable launched and ran healthy (PID 20748, 379 MB RAM, clean shutdown) |

---

## ✦ 4. Defects Resolved
- **[BUG-008] UI Uses Amber/Orange Instead of Target Purple Glass Theme**: Fully resolved and moved to Resolved Bugs in `BUGS.md`.

---

## ✦ 5. Next Steps for Feature Agent (Phase 1D)
- With the central Theme tokens and purple glass styling established, `feature` agent can proceed to **TASK-1D** to implement:
  - 3-column Now Playing layout (Large artwork + Synced lyrics + Audio Analyzer card with Peak/LUFS/Dynamic Range/LR meters).
  - Top bar Back/Forward navigation history logic.
  - Smart playlists (Recently Played, Most Played, Recently Added, Favorites, Never Played, Random Discovery).
