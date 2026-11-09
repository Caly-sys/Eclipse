# Umbra Visual Design Specification (Purple Glass Theme)

Derived from `docs/reference/target-umbra.png`.

---

## 1. Color Palette & Glass Aesthetics

| Token | Hex / RGBA | Role / Usage |
|---|---|---|
| `bg_canvas` | `#0b0914` | Deep obsidian / charcoal-purple background with subtle vignette |
| `panel_surface` | `rgba(20, 16, 34, 0.75)` | Frosted glass panel fill with backdrop blur |
| `card_surface` | `rgba(28, 22, 48, 0.65)` | Sub-card fill for track rows, album art, and cards |
| `panel_border` | `rgba(180, 140, 255, 0.14)` | Hairline luminous translucent border (1.0px) |
| `accent_primary` | `#b388ff` | Radiant neon violet for highlights, active tabs, and primary controls |
| `accent_glow` | `rgba(179, 136, 255, 0.35)` | Specular halo on active buttons, progress sliders, and playing rows |
| `accent_warm` | `#ffb25c` | Secondary warm amber accent for lossless/hi-res badges and peak indicators |
| `text_primary` | `#f4f0ff` | Bright high-contrast text for titles and current lyric line |
| `text_secondary`| `#b2a8cf` | Soft lilac-grey for artists, album names, and section headers |
| `text_muted` | `#6d6488` | Faded lavender for timestamps, inactive lyrics, and metadata tags |

---

## 2. Geometry & Spacing
- **Border Radii**:
  - Window frame: `12.0px`
  - Cards & Panels: `10.0px` - `14.0px`
  - Badges & Pills: `10.0px`
  - Play button: Circular `18.0px` (diameter `36.0px`)
- **Spacing**:
  - Screen edge margins: `10.0px`
  - Card inner padding: `12.0px` to `16.0px`
  - Component gap: `8.0px`

---

## 3. Structural Layout

### Top Titlebar
- **Left**: Minimalist Umbra logo icon + `UMBRA` uppercase bold wordmark + Back/Forward navigation buttons (`‹`, `›`).
- **Center**: Centered translucent search bar with icon `🔍`, placeholder `"Search..."`, and trailing shortcut badge `"Ctrl K"`.
- **Right**: Secondary action icons + Frameless Window controls (`—` minimize, `□` maximize, `✕` close).
- **Behavior**: Draggable anywhere on non-interactive areas.

### Left Sidebar
- Independent vertical scroll area (`ScrollArea::vertical()`).
- Uppercase section headers with muted typography (`size: 10.5`, `tracking: 1.2`):
  - **`LIBRARY`**: Home, Songs, Artists, Albums, Genres, Folders
  - **`SMART`**: Recently Played, Most Played, Recently Added, Favorites, Never Played, Random Discovery
  - **`STREAM`**: Spotify, YouTube Music (rendered with a `(Disabled)` badge)
  - **`PLAYLISTS`**: `+` create button, user playlists with track counts
  - **`ABOUT`**: Placed at bottom without overlapping settings.

### Now Playing Screen (Three-Column Layout)
1. **Left (Track & Art)**:
   - Large rounded album artwork (`260x260px`) with soft ambient purple drop shadow.
   - Title, Artist, Album, Year.
   - Tech Badges: `FLAC`, `24-BIT 48 KHZ`, play counts.
2. **Center (Synced Lyrics)**:
   - Dynamic vertical scroll area.
   - Current line: Bright glowing `#f4f0ff` with accent glow, larger font (`18.0px`).
   - Prior / upcoming lines: Smoothly dimmed and fading (`size: 14.5px`, `#6d6488`).
   - Click to seek.
3. **Right (Audio Analyzer Card)**:
   - Frosted card displaying:
     - `HI-RES LOSSLESS` glowing badge.
     - Codec, Sample Rate, Bitrate, Channels.
     - Real-time Peak, LUFS, Dynamic Range stats.
     - Live L/R level meters.
     - 20 Hz - 20 kHz responsive spectrum visualizer.

### Docked Bottom Playerbar
- **Left**: Mini square album thumbnail (`44x44px`), Title, Artist, Favorite heart toggle, and format badge.
- **Center**: Playback controls (Shuffle, Previous, large circular Play/Pause button, Next, Repeat) and a thin continuous scrubbing seek bar with formatted timestamps (`01:24` / `04:32`).
- **Right**: Mini spectrum bars, Mute button, Volume slider (`0-100%`), EQ button, Lyrics toggle button, and Queue toggle button.
