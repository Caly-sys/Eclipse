# Handoff Report: TASK-2C — Album Cover Art Extraction, Caching, and UI Texture Rendering

- **Task ID**: `TASK-2C`
- **Agent**: `orchestrator`
- **Status**: `HANDOFF_READY`
- **Date**: 2026-11-09

---

## ✦ 1. Summary of Changes

TASK-2C fulfills the user's primary feature request:
> *"can u make it that it shows the cover, and reisizing it is kinda hard rn other tho its fine and Change The FLAC or file format Text on the side of the song much visible"*
> *"makes a agents Do the Sizing and fonts and u do the Cover for the music"*

### 1. Embedded Tag Art Extraction & Disk Caching (`src/library/scanner.rs`)
- Implemented `get_or_extract_cover_art(track_path: &Path) -> Option<Vec<u8>>`:
  1. Checks local persistent disk cache in `%APPDATA%\Umbra\art_cache\<hash>.bin`.
  2. If missing from cache, probes audio tags (FLAC Vorbis comments, ID3v2, MP4/M4A metadata) via `lofty::Probe` and extracts embedded cover artwork binaries.
  3. If no embedded tags exist, scans parent album directory for local artwork image files (`cover.jpg`, `folder.jpg`, `front.jpg`, `album.jpg`, `cover.png`, `folder.png`).
  4. Automatically caches extracted images into `%APPDATA%\Umbra\art_cache\` for instantaneous subsequent frame loads.

### 2. GPU Texture Management & Downsampling (`src/ui/art.rs`)
- Created `AlbumArtManager`:
  - `get_or_load(ctx: &Context, track_path: &str) -> Option<&TextureHandle>`:
    - Decodes images via `image::load_from_memory`.
    - Downsamples large images to a maximum of `512x512` using `dyn_img.thumbnail(512, 512)` to conserve GPU VRAM and maintain steady 60fps rendering.
    - Uploads `egui::ColorImage` into GPU linear-filtered textures (`TextureOptions::LINEAR`).
    - Caches texture handles in memory (`HashMap<String, TextureHandle>`). Subsequent 60 FPS frames perform an instantaneous hash map lookup without disk or CPU overhead.
    - Tracks failed files in `HashSet<String>` to prevent redundant I/O retries.

### 3. Now Playing Album Art Display (`src/ui/views/now_playing.rs`)
- Replaced the placeholder emoji with full GPU-accelerated cover art:
  - Renders the cover within a squared card (`card_width` between 180px and 260px depending on viewport width).
  - Preserves ambient specular violet glow halos behind the artwork.
  - Draws a hairline translucent border (`theme.border_accent`) with 14px rounded corners.
  - Gracefully falls back to the centered `🎵` placeholder icon if no artwork is embedded or found in the album directory.

### 4. Mini Cover in Docked Playerbar (`src/ui/playerbar.rs`)
- Replaced the placeholder emoji in the 44x44 mini playerbar thumbnail:
  - Renders active track's cover image with 8px rounded corners.
  - Clicking either the cover art or track title toggles the full Now Playing screen.
  - Falls back to `🎵` placeholder when no track is loaded or art is absent.

### 5. Albums Grid & Details View (`src/ui/views/albums.rs`)
- **Album Grid**:
  - Each 150x190 album card displays the cover image of the album's first track in the 150x130 upper artwork region with 8px rounded borders.
- **Album Details Header**:
  - The 100x100 artwork hero box displays the high-resolution album cover texture next to the album title, artist, year, and "▶ Play Album" button.

### 6. Home View Highlights (`src/ui/views/home.rs`)
- Favorites quick-play cards display a 36x36 cover art thumbnail.
- Recently Played track list displays a mini 24x24 cover art thumbnail next to each track.

---

## ✦ 2. Files Modified & Created

| File | Change |
|---|---|
| [`src/library/scanner.rs`](file:///C:/music/umbra/src/library/scanner.rs) | Added `get_or_extract_cover_art`, disk caching helpers, and tag probing |
| [`src/ui/art.rs`](file:///C:/music/umbra/src/ui/art.rs) | **New file**: `AlbumArtManager`, GPU texture caching, downsampling, and unit tests |
| [`src/ui/mod.rs`](file:///C:/music/umbra/src/ui/mod.rs) | Registered `pub mod art;`, added `art_manager` field to `UmbraApp`, and routed to views |
| [`src/ui/views/now_playing.rs`](file:///C:/music/umbra/src/ui/views/now_playing.rs) | Rendered album cover texture in Now Playing screen |
| [`src/ui/playerbar.rs`](file:///C:/music/umbra/src/ui/playerbar.rs) | Rendered 44x44 mini cover art thumbnail in docked playerbar |
| [`src/ui/views/albums.rs`](file:///C:/music/umbra/src/ui/views/albums.rs) | Rendered cover art in album grid cards and album header card |
| [`src/ui/views/home.rs`](file:///C:/music/umbra/src/ui/views/home.rs) | Rendered cover art in Favorites and Recently Played cards |
| [`TASKS.md`](file:///C:/music/umbra/TASKS.md) | Updated `TASK-2C` status to `HANDOFF_READY` |

---

## ✦ 3. Build & Test Verification

- **`cargo check`**: PASSED (0 errors, 0 warnings).
- **`cargo clippy`**: PASSED (0 errors, 0 warnings).
- **`cargo test`**: PASSED (26/26 tests passing in 0.09s).
  - `ui::art::tests::test_art_manager_initialization`: PASSED
  - `ui::art::tests::test_art_manager_failed_caching`: PASSED
  - `ui::art::tests::test_synthetic_image_decoding`: PASSED
