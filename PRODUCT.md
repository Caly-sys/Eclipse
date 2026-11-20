# Product Specification: Eclipse Music Player

## 1. Vision & Core Value
Eclipse is a fast, local-first lossless music player and streaming client for Windows and desktop OSes. Built in Rust with `egui` and `rodio`/`symphonia`, it provides audiophile audio playback (bit-perfect FLAC, DSD, MP3, AAC, ALAC, Opus), synchronized lyrics (LRCLIB), offline library indexing via SQLite, online artwork enrichment, and seamless streaming integrations.

## 2. Target Persona & Use Cases
- **Audiophile Collectors**: Users with extensive local audio libraries who demand bit-perfect output, Hi-Res audio badges, accurate sample rate/bitrate metadata, and gapless/equal-power playback.
- **Desktop Power Listeners**: Users who keep Eclipse open on a secondary monitor or floating in PiP while working, desiring high-density scanability, rich synced lyrics, and instant queue inspection without menu diving.
- **Aesthetic Purists**: Users seeking dark-mode refinement inspired by Feishin, Plexamp, and macOS desktop apps—tonal depth, muted glass surfaces, precise typography, and zero visual clutter.

## 3. UI/UX Paradigm: Feishin 3-Column Bento Architecture
- **Mode**: Operate (high task density, scannable queues and library grids, crisp information hierarchy, 8px layout rhythm).
- **Column 1 (Left Rail)**: Collapsible navigation rail (220px expanded, 64px icon-only rail). Fast switching between Library, Smart playlists, Streams, and User playlists.
- **Column 2 (Center Stage)**: Dedicated main workspace for browsing Albums, Artists, Songs, Genres, Folders, and Settings, framed inside an elevated bento panel.
- **Column 3 (Right Inspector)**: Persistent 300–320px inspector panel featuring top Now Playing block, 3-way segmented switcher (`Queue` | `Lyrics` | `Details`), live synced lyrics, and interactive upcoming queue.
- **Floating Bottom Player Dock**: Unified 3-zone audio control dock hovering over the canvas with album art, scrubbable seekbar, playback transport, volume control, and quick inspector toggles.

## 4. Key Quality Constraints
- **Zero Panics**: No `.unwrap()` or `.expect()` in user interface code.
- **Zero Emojis**: 100% pure vector iconography via `Icon` renderers.
- **Performance**: Instant 60 FPS egui rendering, minimal allocation overhead, smooth scrolling, and non-blocking background queue prefetching.
