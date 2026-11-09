# Umbra Manual Testing & Quality Verification Guide

This checklist covers all interaction points, window controls, playback sliders, keyboard shortcuts, 3-column Now Playing view, smart playlists, and typography in Umbra. Verified on Windows with `umbra.exe` release build by `qa-reviewer`.

---

## 1. Window Controls & Titlebar

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **WB-01** | Titlebar Window Drag | Click and drag with Primary mouse button on empty areas or wordmark in the custom titlebar. | Window moves fluidly following cursor across the screen. | **PASS** | Triggered via `ViewportCommand::StartDrag` |
| **WB-02** | Titlebar Double-Click | Double-click on any empty area of the titlebar. | Window toggles between Maximized and Restored size. | **PASS** | Toggles `ViewportCommand::Maximized` cleanly |
| **WB-03** | Minimize Button (`–`) | Click the minimize button in the top-right corner. | Window minimizes smoothly to the Windows taskbar. Hover shows subtle translucent highlight. | **PASS** | `ViewportCommand::Minimized(true)` |
| **WB-04** | Maximize / Restore Button (`□` / `❐`) | Click the maximize button; click again when maximized. | Window maximizes to fill screen, icon switches to `❐`. Clicking again restores window, icon switches back to `□`. | **PASS** | Correctly detects state and switches icons |
| **WB-05** | Close Button (`✕`) | Hover over close button; click it. | Button highlights with red background (`#dc2626`) on hover. Clicking exits the application cleanly. | **PASS** | Red hover verified; clean exit |
| **WB-06** | Border Edge Resizing | Hover and drag within 7px of any window edge (top, bottom, left, right). | Cursor changes to directional resize cursor (`ResizeNorth`, `ResizeSouth`, `ResizeWest`, `ResizeEast`); dragging resizes window via OS sizing loop. | **PASS** | Triggered via `ViewportCommand::BeginResize` |
| **WB-07** | Corner Resizing | Hover and drag within 7px of any of the 4 window corners. | Cursor changes to diagonal resize cursor (`ResizeNorthWest`, `ResizeNorthEast`, `ResizeSouthWest`, `ResizeSouthEast`); dragging smoothly resizes width & height simultaneously. | **PASS** | Corners take priority over edges |
| **WB-08** | Bottom-Right Resize Grip Handle | Hover and drag the diagonal `///` glass grip lines in the bottom-right corner of the docked playerbar. | Cursor changes to `ResizeSouthEast`, lines highlight in violet accent, tooltip displays `"Drag to resize window"`, dragging smoothly resizes window. | **PASS** | Visible visual affordance in bottom-right playerbar |

---

## 2. Interactive Seek Bar & Timeline

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **SK-01** | Single-Click to Seek | While a song is playing, click directly on the middle or end of the seek bar rail. | Playback jumps immediately to the clicked timestamp without requiring a drag. Progress rail updates instantly. | **PASS** | Direct jump computed from pointer position |
| **SK-02** | Continuous Scrubbing | Click and hold on the seek bar, dragging horizontally left and right. | Progress rail and scrubber knob smoothly track mouse pointer; audio seeks smoothly without jitter or stutter. | **PASS** | Continuous seeking tracks pointer without jitter |
| **SK-03** | Hover Time Preview | Hover mouse cursor over the seek bar without clicking. | Cursor changes to pointing hand (`PointingHand`), track rail expands from 4px to 6px, glowing knob appears, and tooltip displays the exact `mm:ss` timestamp. | **PASS** | Rail expansion, knob halo, and tooltip active |
| **SK-04** | Large Hit Target | Click slightly above or below the thin 4px rail (within the 18px hit target). | Seek registers reliably without requiring pixel precision. | **PASS** | 18px hit target allocated |

---

## 3. Volume Slider & Mute Toggle

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **VL-01** | Direct Volume Jump | Click directly at 25%, 50%, or 80% of the volume bar. | Volume jumps immediately to that level. Volume knob appears with percentage tooltip (`50%`). | **PASS** | Instant level change and percentage display |
| **VL-02** | Smooth Volume Dragging | Drag the volume slider left and right. | Volume adjusts continuously from 0% to 100%. | **PASS** | Smooth linear volume response |
| **VL-03** | Mouse Wheel Volume Scroll | Hover mouse over the volume bar and scroll the wheel up and down. | Volume increments / decrements in ±5% steps. | **PASS** | `raw_scroll_delta.y` adjusts in ±0.05 steps |
| **VL-04** | Mute Toggle Button | Click the mute icon button (`🔊`). | Audio mutes, icon changes to `🔇`. Clicking again unmutes and restores previous volume level. | **PASS** | Previous volume remembered on unmute |
| **VL-05** | Auto-Unmute on Volume Increase | While muted, drag the volume slider or scroll wheel up. | Audio automatically un-mutes and sets the new volume level. | **PASS** | Un-mutes when new_vol > 0 |

---

## 4. Global Keyboard Shortcuts

| ID | Shortcut | Context | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **KB-01** | `Space` | Playback active | Toggles between Play and Pause. Toast confirms `"Playing"` or `"Paused"`. | **PASS** | Instant toggle with toast feedback |
| **KB-02** | `Space` | Stopped with queue | Starts playback of first track in queue. Toast confirms `"Playing queue"`. | **PASS** | Starts track 0 in queue |
| **KB-03** | `←` (Left Arrow) | Song playing | Seeks backward by 5 seconds. Toast displays `"Seek -5s [mm:ss]"`. | **PASS** | Clamped at 0.0s |
| **KB-04** | `→` (Right Arrow) | Song playing | Seeks forward by 5 seconds. Toast displays `"Seek +5s [mm:ss]"`. | **PASS** | Clamped at track duration |
| **KB-05** | `↑` (Up Arrow) | Anywhere | Increases volume by +5%. Toast confirms `"Volume: XX%"`. | **PASS** | Auto-unmutes if muted |
| **KB-06** | `↓` (Down Arrow) | Anywhere | Decreases volume by -5%. Toast confirms `"Volume: XX%"`. | **PASS** | Clamped at 0% |
| **KB-07** | `M` | Anywhere | Toggles mute state. Toast displays `"Muted"` or `"Unmuted (XX%)"`. | **PASS** | Mute toggle working |
| **KB-08** | `S` | Anywhere | Toggles shuffle. Shuffle button lights up accent; toast confirms `"Shuffle: ON"` / `"OFF"`. | **PASS** | Shuffle indices regenerated on toggle |
| **KB-09** | `R` | Anywhere | Cycles repeat mode (Off → All → One). Button icon updates; toast confirms `"Repeat: ALL"` / `"ONE"` / `"OFF"`. | **PASS** | Cycles through all 3 modes |
| **KB-10** | `Ctrl + K` | Anywhere | Focuses the top search box with text cursor active and ready to type. | **PASS** | `focus_search` triggers `request_focus()` |
| **KB-11** | `F11` | Anywhere | Toggles borderless fullscreen mode. | **PASS** | `ViewportCommand::Fullscreen` |
| **KB-12** | Text Input Guard | Typing in search box | Pressing `Space`, `M`, `S`, `R`, or Arrow keys types characters and moves cursor — does NOT trigger playback or shortcuts. | **PASS** | `wants_keyboard_input()` guard suppresses playback keys |

---

## 5. Sidebar & Navigation Scrolling

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **SB-01** | Tab Switching | Click each sidebar tab: Home, Now Playing, Songs, Artists, Albums, Genres, Folders, Playlists, Settings, About. | View switches instantly with smooth active border and accent background. | **PASS** | Active pill indicator + neon glow |
| **SB-02** | Independent Sidebar Scroll | Resize window height down to minimum (600px). Scroll sidebar vertically. | Sidebar navigation buttons scroll smoothly within their scroll area. | **PASS** | Independent `ScrollArea::vertical()` |
| **SB-03** | No Stats Collision | Inspect bottom of sidebar at 600px window height. | The `📚 XX Tracks / XX Playlists` card remains neatly positioned and never collides with or obscures the Settings button. | **PASS** | Linear flow without bottom_up collisions |
| **SB-04** | Sidebar Section Structure | Inspect sections in sidebar. | Hierarchy reflects `LIBRARY`, `SMART`, `STREAM`, `PLAYLISTS` (with `+` button), `PREFERENCES`. | **PASS** | Clean muted headers, stream items disabled |

---

## 6. Song Table Interaction

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **ST-01** | Single-Click Row Selection | Click once on a song row in the Songs view. | Row becomes visually selected with accent tint and border; current playback does not stop. | **PASS** | `selected_track_id` highlighted |
| **ST-02** | Double-Click Row Playback | Double-click quickly on any song row. | The entire filtered track list loads into queue and the selected song begins playing immediately. | **PASS** | Loads filtered queue at clicked index |
| **ST-03** | Row Hover Feedback | Move cursor across song rows. | Pointing hand cursor is displayed; hovered row highlights with subtle light surface. | **PASS** | Cursor and hover background active |
| **ST-04** | Favorite Toggle Isolation | Click the heart icon (`❤️` / `🤍`) on a song row. | Favorite status toggles in SQLite database; the song does NOT start playing from single-clicking the heart. | **PASS** | Heart click isolated with `fav_clicked` flag |
| **ST-05** | Recently Played Playback | In Home view, click on any track card under "Recently Played". | Track begins playing immediately. | **PASS** | Triggers queue playback |
| **ST-06** | Now Playing Toggle | In playerbar, click the song title or album thumbnail. | Toggles to Now Playing view. | **PASS** | `toggle_now_playing` switches to `NowPlaying` |

---

## 7. Now Playing 3-Column View

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **NP-01** | 3-Column Audiophile View | Open Now Playing during playback. | Left: Artwork Card; Center: Synced Lyrics; Right: Audio Analyzer Card. | **PASS** | `ui.columns(3)` layout |
| **NP-02** | Artwork Card & Badges | Inspect Column 1 (Left). | Large rounded card with ambient violet glow halo, Title, Artist, Album/Year, badges (`FLAC`/format, bit depth/sample rate, play count, lossless/hi-res). | **PASS** | Halo glow + warm amber `#ffb25c` badges |
| **NP-03** | Synced Lyrics & Click-to-Seek | Inspect Column 2 (Center). | Synced lyrics auto-scroll to active line. Current line is bright `#f4f0ff` with accent glow; inactive lines dimmed in `#6d6488`. Clicking line seeks playback. | **PASS** | Auto-scroll via `scroll_to_me(Center)`; click-seek works |
| **NP-04** | Audio Analyzer Card | Inspect Column 3 (Right). | Glowing `HI-RES LOSSLESS` / `LOSSLESS` badge in warm amber, technical specs grid, Peak dBFS / LUFS / DR metrics, animated L/R meters, 20Hz-20kHz spectrum visualizer with style toggles (`Bars`, `Mirror`, `Scope`). | **PASS** | Real-time stereo meters & frequency axis markers |

---

## 8. Navigation & History

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **NV-01** | History Back & Forward | Navigate Songs → Artists → Albums, then click `‹` Back, then `›` Forward. | Back returns to Artists (Forward enables); Forward returns to Albums. Buttons visually enable/disable. | **PASS** | Stack pointer tracking verified |
| **NV-02** | Branch Truncation | Go back to Songs, then click Genres. | Prior forward history (Artists, Albums) is truncated and Genres becomes head. | **PASS** | `nav_history.truncate(nav_history_idx + 1)` |
| **NV-03** | Multi-View Instant Search | Enter text in titlebar search bar. | Instantly filters Songs, Albums, Artists, and Genres. Pressing Enter from other tabs navigates to Songs. | **PASS** | Real-time substring filtering across fields |

---

## 9. Typography, CJK Glyph Rendering & Purple Glass Aesthetic

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **TR-01** | CJK Zero-Tofu Rendering | Display Japanese tracks (Aimyon, Yorushika, Eve, Kenshi Yonezu) and lyrics. | Kanji, Hiragana, and Katakana glyphs render crisply without empty boxes (`[]`). | **PASS** | Windows CJK fallback fonts loaded |
| **TR-02** | Purple Glass Aesthetic Adherence | Inspect UI against `docs/reference/target-umbra.png`. | Canvas `#0b0914`, frosted panels `#141022`, translucent borders `rgba(180, 140, 255, 0.14)`, neon violet `#b388ff`, warm amber `#ffb25c` badges. | **PASS** | Central tokens in `Theme` match target |

---

## 10. Phase 2: High-Contrast Format Badges & Table Alignment

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **FB-01** | Lossless Format Badges (FLAC/WAV/ALAC) | View any lossless track in Songs, Now Playing, or Playerbar. | Renders in bold warm neon amber (`#ffb25c`) with luminous 22% fill and crisp 65% amber border. High contrast and immediately legible. | **PASS** | Verified in `src/ui/components.rs` via `format_badge` |
| **FB-02** | Lossy Format Badges (MP3/AAC/OGG) | View any lossy track in Songs, Now Playing, or Playerbar. | Renders in bright high-contrast lilac (`#ebe4ff`) with frosted dark glass fill and defined 35% lavender border. | **PASS** | High contrast on all theme backgrounds |
| **FB-03** | Song Table Column Alignment | Inspect Songs table columns at multiple window sizes. | Format badges are centered directly under the `"FORMAT"` header (60px slot); duration timestamps are right-aligned under `"TIME"` (48px slot). | **PASS** | Symmetric right-to-left layout in header and rows |
| **FB-04** | Duration Timestamp Typography | Inspect song duration timestamps across table rows. | Upgraded from dark muted text to bright `theme.text_secondary` (`#b2a8cf`). | **PASS** | High-contrast duration display |

---

## 11. Phase 2: Album Cover Art Extraction, Caching & Rendering

| ID | Test Case | Action | Expected Result | Pass/Fail | Notes |
|---|---|---|---|---|---|
| **CA-01** | Embedded Tag Art Extraction | Scan local music folder `C:\music\Muse`. | Extracts embedded album art from audio tags (FLAC Vorbis, ID3v2, MP4) and local folder images (`cover.jpg`/`folder.jpg`). | **PASS** | 68 unique cover art files extracted & cached |
| **CA-02** | Persistent Disk Caching | Inspect `%APPDATA%\Umbra\art_cache\`. | Cover images are cached as SHA256 hashed `.bin` files for instantaneous subsequent load. | **PASS** | 68 `.bin` files cached, validated as JPEG/PNG |
| **CA-03** | GPU Texture Downsampling & Memory | Check memory and frame rate during cover art rendering. | Images downsampled to max 512x512 thumbnail, linear texture filtering, cached in `AlbumArtManager`. Memory steady at ~377 MB. | **PASS** | Texture handles cached, no GPU thrashing |
| **CA-04** | Now Playing Cover Art Card | Open Now Playing during track playback. | Displays real high-resolution album cover art in squared card with ambient specular violet glow halo. Falls back to `🎵` if missing. | **PASS** | Full cover rendered with rounded 14px corners |
| **CA-05** | Playerbar Mini Cover Thumbnail | Inspect bottom docked playerbar during playback. | Displays 44x44 mini album cover thumbnail with 8px rounded corners. Clicking toggles Now Playing. | **PASS** | High-resolution thumbnail with click toggle |
| **CA-06** | Albums Grid & Details View Art | Navigate to Albums view; click any album card. | Album grid cards display 150x130 cover art; Album details header displays 100x100 high-res cover art. | **PASS** | Grid and header view cover art active |
| **CA-07** | Home View Cover Art | Navigate to Home view. | Favorites cards display 36x36 cover thumbnails; Recently Played rows display 24x24 cover thumbnails. | **PASS** | Instant cover display across Home view |
