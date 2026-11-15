## ✦ Eclipse v1.2.4 — Streaming & Download Update

Eclipse v1.2.4 delivers a major streaming and download overhaul, featuring a whole playlist/album batch downloader with a bounded concurrency queue, an audio format selector (FLAC, MP3, OPUS, M4A, WAV), key-less audio resolution, top-right download queue and history UI, silenced console windows, and a complete Windows system tray lifecycle overhaul.

### ✦ Highlights & Fixes

- **Whole Playlist / Album Batch Downloader**:
  - Download entire Spotify playlists and albums in batch with one click via `[ ↓ Download All (N) ]`.
  - Added `[ ↓ Download Missing (N) ]` to local saved playlists to batch-download remote stream tracks.
  - Automatic upgrade: downloaded tracks are tagged with lofty and indexed into SQLite, automatically upgrading saved playlist entries from `STREAM` to `LOCAL`.
- **Bounded Concurrency Queue (`MAX_CONCURRENT_DOWNLOADS = 2`)**:
  - Queue up 50+ tracks safely without spawning uncontrolled worker threads or spiking system resources.
  - Automatically progresses through queued tracks upon completion, failure, or cancellation.
  - Added batch controls: **"Cancel All"** for active queues and **"Retry Failed"** for interrupted downloads.
- **Multi-Format Audio Selector (FLAC, MP3, OPUS, M4A, WAV)**:
  - Choose your preferred download format directly in the Downloads drawer (`FORMAT [ FLAC ] [ MP3 ] [ OPUS ] [ M4A ] [ WAV ]`) or via the Spotify action bar.
  - Formats persist across sessions in settings.
  - Full metadata and album art tagging via `lofty` with real `sample_rate` and `bit_depth` probed immediately into SQLite.
- **Top-Right Download Queue & History UI**:
  - Added a `Downloads` pill button in the top bar with active job count badge e.g. `Downloads (1)`.
  - Floating glass popover with separate **Queue** (progress bar, status, cancel) and **History** (completed tracks with "Play" & "Open Folder", failed tracks with error reasons & "Retry").
- **Key-Less Audio Resolution**:
  - No YouTube Data API key required for downloads or stream playback.
  - Resilient fallback hierarchy: SQLite cache → YouTube Data API v3 (if key set) → `yt-dlp` CLI fallback → direct YouTube web scrape.
- **Zero Windows CMD Popups (`CREATE_NO_WINDOW`)**:
  - All background processes (`yt-dlp`, `where.exe`, etc.) are spawned with `CREATE_NO_WINDOW` and null stdin, eliminating all flashing console windows on Windows.
- **Stream Playback Freeze/Crash Fix & Seamless Hand-Off**:
  - Non-blocking stream resolution with local temporary caching (`eclipse_stream_cache`).
  - Seamless hand-off directly into Eclipse's Symphonia audio engine: timeline seeking, volume leveling, Rodio tap visualizers, and auto-advance work identically to local tracks.
  - Non-blocking `"Buffering stream..."` indicators in the playerbar and tracklist.
- **Window Close & Background System Tray Lifecycle**:
  - Fixed Windows `MAIN_HWND` resolution: window close (`×`) now cleanly hides the true native OS window to the system tray when music is playing (`ShowWindow(hwnd, SW_HIDE)`).
  - Clean exit when playback is paused or stopped, preventing uncloseable window states.
  - Automatically dismisses the background status banner whenever the window is in the foreground.
- **High-Contrast Track Row Styling**:
  - Replaced low-contrast solid lavender highlight with a rich, dark translucent purple tint (`Color32::from_rgba_premultiplied(124, 77, 255, 35)`), maintaining crisp white text contrast at all times.

### ✦ SHA-256 Checksums (v1.2.4)

| File | SHA-256 Checksum |
|---|---|
| `eclipse.exe` | `6C0DCFD803AA911B4FB8978BD6F6800F2310B67E1BE72426717B5E7B4EA59B31` |
| `eclipse-v1.2.4-windows-x64.zip` | `6BA47902BA7F9970BE28C0FC983EA076319F88255D57DDC47ADEBF72599A7D93` |

### ✦ Installation
- Download and extract `eclipse-v1.2.4-windows-x64.zip` or run `eclipse.exe` directly.
- No developer keys or accounts required; paste any public Spotify link to import, stream, or download.
