## ✦ Eclipse v1.2.6 — The UI Rework & Japanese Romaji Lyric Update

### 🌸 Context-Aware Japanese Romaji Lyrics
- **Offline Morphological Romaji Engine**: Integrated `lindera` tokenizer with embedded IPADIC dictionary for context-aware transliteration of Kanji, Hiragana, and Katakana into natural Hepburn Romaji.
- **Synced Lyrics & Fullscreen Sub-Lines**: Smooth Romaji sub-lines across Now Playing, Fullscreen mode, and Right Inspector, cached once per track with zero per-frame CPU overhead.
- **Smart Language Detection**: Automatically hides the `Romaji` toggle button on English, Korean, and other non-Japanese songs.
- **Seamless Per-Line Edge Alpha Dissolve**: Replaced overlay fade meshes with true per-line alpha opacity modulation for zero-seam edge dissolution on any theme with zero sliced characters.
- **Frameless Cover Art & Window Border Polish**: Refined cover art dimensions with soft ambient drop shadow and subtle 1px border.

### 🍱 Feishin-Style Matte-Slate Bento 3-Column Architecture
- **Layered Matte Charcoal Surfaces**: Completely overhauled visual architecture replacing wireframe borders with a refined matte surface hierarchy (`#12151A` bar surfaces, `#161A20` sidebars & inspector, `#1D2129` center viewport, and `#252A34` elevated cards).
- **Subtle 1px Structural Pane Seams**: Replaced harsh high-contrast white strokes with quiet, dark 1px pane dividers (`Color32::from_rgb(36, 41, 50)`) for crisp architectural separation.
- **Dynamic Bento Palette & Settings Theming**: Dynamic palette generator powered by active Settings Themes (`Ember`, `Verdant`, `Nebula`, `Glacier`, `Obsidian`, `Eclipse`) and Accent Colors.

### 🌌 Edge-to-Edge Home Ambient Gradient
- **Full-Bleed Center Glow**: Edge-to-edge 10-row multi-stop `egui::Mesh` blending from center slate into active theme accent color across the entire top header width with zero dark gutters.
- **Ease-Out Color Mathematics**: Calibrated lerp curve maintains pure RGB color vibrancy without dark grey muddying.

### 🎵 Restored Now Playing View & Enhanced Right Inspector
- **Real-Time PCM Spectrum Analyzer**: Restored dedicated Now Playing view with logarithmic FFT frequency visualizer and stereo peak meters.
- **LRCLIB Cloud Lyrics & Manual Search**: Interactive `Search Online (LRCLIB)` modal to query, preview, and apply synced lyrics on the fly.
- **High-Contrast Active Lyric Lines**: Enhanced readability across dark matte surfaces with crisp translucent highlight pill and gold badges.

### 🪟 Native Windows Pop-Out Mini-Player Rounding
- **DWM Native Window Rounding**: Native Win32 `DwmSetWindowAttribute` / `CreateRoundRectRgn` rounded corners on the borderless mini-player viewport (`HWND`), eliminating dark rectangular corner artifacts on Windows 10/11.

### 🛡️ Backend Reliability & Race Condition Fixes
- **Track Skipping Lyrics Race Elimination**: Generational request ID tracking on async LRCLIB requests ensures late-arriving lyrics from skipped tracks never overwrite the active track.
- **Downloader Mutex Poisoning Recovery**: All download worker queues safely recover from poisoned mutex state instead of crashing the process on panic.
- **Safe HTML Entity Decoding**: Hardened entity parsing for YouTube metadata strings.

---
**Binary Verification:**
- `SHA256(eclipse.exe) = CC65B7D1C84B0680C8441300E80AA3F6600677E73B6659F08A237658613062EB`
- `SHA256(eclipse-v1.2.6-windows-x64.zip) = 5C23F2E412266D0A9ED8FA29E3374277D5D1F5D6BC1317D296553B5C0C97D7DF`
