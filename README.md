# Eclipse — music, in the dark.

A high-performance, local-first lossless music player for Windows rewritten in **pure Rust** with a calm, dark purple glass aesthetic.

---

## ✦ Preview

### Home Dashboard
*Bento grid layout featuring "Pick up where you left off", real-time listening metrics, on-repeat tracks, and horizontal album shelf.*

![Eclipse Home Dashboard](IMG/ss1.png)

<br/>

### Albums Library
*Responsive auto-wrapping grid with high-resolution embedded cover art decoding, clean typography, and 1:1 aspect ratios.*

![Eclipse Albums Grid](IMG/ss2.png)

<br/>

### Playlist & Multi-Column Track Table
*Custom playlists with hero header, duration metrics, and clean multi-column table displaying format badges, artist, album, and relative timestamps.*

![Eclipse Playlist View](IMG/ss3.png)

---

## ✦ Highlights

- **Rust Audio Engine**: Powered by `rodio` and `symphonia` with real-time PCM audio stream tapping. Zero audio stuttering with an 8192-sample non-blocking buffer.
- **Studio-Grade Real-Time Visualizer**:
  - 2048-sample forward Hann-windowed FFT via `rustfft`.
  - Logarithmic frequency mapping from `20Hz` to `20kHz` with +3dB/octave pink-noise compensation and dBFS temporal smoothing/gravity decay.
  - Real-time time-domain oscilloscope buffer and live Left/Right channel RMS & Peak dBFS meters. Zero fake math waves.
- **100% Vector Icon Architecture**:
  - Completely free of Unicode emoji and symbol placeholders.
  - 46 custom vector icons (Lucide / Feather style) rendered crisply directly on `egui::Painter` with dynamic DPI scaling.
- **Glassmorphic Purple UI**:
  - Centralized design tokens in `src/ui/theme.rs`.
  - Obsidian canvas (`#0b0914`), translucent frosted glass cards, neon violet accent (`#b388ff`), and warm amber lossless format badges (`#ffb25c`).
- **Responsive Layout & Dynamic Grids**:
  - Fixed 240px left sidebar with smooth independent scrolling and full-width library counter.
  - Responsive auto-wrapping grid (`minmax(165px, 1fr)`) for Albums, Artists, and Genres with strict 1:1 square cover art aspect ratios.
  - Full Eclipse Home dashboard with Quick Actions (`Play All`, `Shuffle Library`, `Scan Folder`), 4-card statistics grid, and Recently Played shelf.
  - 3-column docked Player Bar with album art snippet, transport controls, continuous scrubbing seekbar, and mini spectrum visualizer.
- **Full Queue Progression**:
  - Sequential playback, non-repeating shuffle permutation, and repeat modes (`Off`, `All`, `One`).
  - Seamless automatic track advancement on EOF.
- **Cover Art & CJK Typography**:
  - Embedded metadata picture extraction via `lofty` with asynchronous decoding on background worker threads.
  - Zero UI thread blocking.
  - Native Windows CJK font fallbacks for Japanese, Korean, and Chinese tracks and synced lyrics.
- **Frameless Window Management**:
  - Custom titlebar drag region, double-click maximize/restore, minimize, and close.
  - 7px border margin hit-testing for 8-direction OS window resizing and bottom-right glass grip handle.

---

## ✦ Installation & Setup

Eclipse is 100% local-first and portable. It requires no administrative installers, registry entries, or background updater daemons.

### Option 1: Standalone Portable Executable (Recommended)

1. Download the latest standalone release binary (`eclipse.exe` or `eclipse-windows-x64.zip`) from [GitHub Releases](https://github.com/Caly-sys/Eclipse/releases).
2. Move `eclipse.exe` into any preferred directory (for example `C:\Tools\Eclipse` or `%LOCALAPPDATA%\Programs\Eclipse`).
3. Double-click `eclipse.exe` to launch.
   - **Tip:** Right-click `eclipse.exe` → **Show more options** → **Send to** → **Desktop (create shortcut)** for easy access from your desktop or taskbar.

> [!NOTE]
> **Windows Defender SmartScreen Notice:**
> Because Eclipse is a community open-source binary without an expensive commercial code-signing certificate, Windows SmartScreen may display a *"Windows protected your PC"* popup on first launch. Click **More info** → **Run anyway** to start the application.

---

### Option 2: Building from Source

If you prefer compiling directly from the Rust source code:

#### Prerequisites
- **Operating System:** Windows 10 or 11 (64-bit)
- **Rust Toolchain:** [rustup.rs](https://rustup.rs/) (Rust 1.80+ recommended, edition 2021)
- **C++ Build Tools:** Visual Studio Build Tools with C++ workload (standard for MSVC ABI)
- **Git:** [git-scm.com](https://git-scm.com/)

#### Clone & Compile
```powershell
# 1. Clone the repository
git clone https://github.com/Caly-sys/Eclipse.git
cd Eclipse

# 2. Build the optimized release binary
cargo build --release

# 3. Launch Eclipse
.\target\release\eclipse.exe
# or use the bundled quick-launch script:
.\run.bat
```

#### Running Tests & Code Quality Checks
```powershell
# Run all unit and integration tests (102 tests)
cargo test --all

# Run strict Clippy linter
cargo clippy --all-targets -- -D warnings
```

---

### Data & Configuration Paths

Eclipse keeps all user configuration, local SQLite indexing, and cached lyrics isolated inside your user profile:

| Path | Description |
|---|---|
| `%APPDATA%\Eclipse\settings.json` | Persistent user settings: audio outputs, DSP crossfade, transitions, volume leveling, monitored folders, and theme preferences. |
| `%APPDATA%\Eclipse\library.db` | High-performance SQLite database storing indexed tracks, album metadata, listening stats, and playlists. |
| `%APPDATA%\Eclipse\lyrics_cache\` | Offline disk cache for plain and synchronized `.lrc` lyrics fetched from LRCLIB. |

To completely reset the player to a fresh state, simply exit the application and delete the `%APPDATA%\Eclipse` directory.

---

## ✦ Usage

### 1. Launching Eclipse
- Run `./run.bat` or `./eclipse.exe` directly.
- The player launches in a frameless glass window. Drag anywhere on the empty titlebar to move the window, or drag borders/corners to resize.

### 2. Scanning Your Music Library
- On the **Home** dashboard, click **Scan Folder** in the Quick Actions banner, or navigate to **Settings** (`⚙ Settings` in sidebar) → **Library Monitored Folders** and click **Add Folder**.
- Select the folder containing your audio files (`FLAC`, `MP3`, `WAV`, `OGG`, `AAC`, `M4A`).
- Eclipse will crawl your files, extract metadata (titles, artists, albums, codecs, sample rates) via `lofty`, cache embedded cover art asynchronously, and store the indexed library in SQLite (`%APPDATA%\Eclipse\library.db`).

### 3. Browsing Your Music
- **Home**: Dashboard showing library statistics, quick actions, Recently Played shelf, and top played tracks.
- **Songs**: High-density sortable list with track titles, artists, albums, duration, and high-contrast lossless format badges (`FLAC` in warm amber, lossy in lilac).
- **Albums**: Responsive auto-wrapping grid of square album covers. Click an album to open its detail view and tracklist.
- **Artists**: Grid of artists with track counts.
- **Folders**: Hierarchical tree navigation through your indexed folders on disk.
- **Smart Playlists**: Instant views for `Recently Played`, `Most Played`, `Recently Added`, `Favorites`, `Never Played`, and `Random Discovery`.
- **Search (`Ctrl + K`)**: Instant global search across tracks, artists, and albums.

### 4. Playback & Queue Controls
- Double-click any song or click **Play All** on an album or view to start playback. This populates the active queue.
- **Play / Pause**: Click the large circular transport button or press `Space`.
- **Next / Previous**: Skip between tracks with transport buttons. If in shuffle mode, tracks play in random non-repeating order while keeping the current song pinned.
- **Repeat Modes**: Cycle between Repeat Off, Repeat All (loops playlist), and Repeat One (loops active track).
- **Seeking**: Click or drag directly on the seekbar for smooth scrub preview.
- **Volume**: Use the volume slider or mouse scroll wheel over the volume area.
- **Queue Drawer**: Click the queue icon button in the player bar to view the upcoming queue, jump directly to any track, or view song details.

### 5. Synced Lyrics & Audio Analyzer
- Switch to the **Now Playing** tab (or click the album art thumbnail in the bottom player bar) to enter the 3-column view:
  - **Left**: Large rounded cover art with ambient glow and track metadata badges.
  - **Center**: Synced `.lrc` lyrics with real-time autoscrolling. The active line glows bright; past and future lines fade gently. Click any lyric line to jump directly to that timestamp.
  - **Right**: Real-time hardware Audio Analyzer card showing format specifications, live Left/Right channel stereo peak meters, integrated LUFS, crest factor dynamic range, and a live 20Hz–20kHz logarithmic spectrum visualizer (switchable between Bars, Mirror, and Oscilloscope).

---

## ✦ Streaming Integrations (Spotify & YouTube Music)

Starting in **v1.2.2**, Eclipse integrates official streaming support for **Spotify** and **YouTube Music** alongside your local lossless library, without proprietary binaries, scrapers, or security vulnerabilities.

### Capabilities Matrix: What Streams Can and Can't Do

| Feature / Capability | Local Lossless Playback | Spotify Stream | YouTube Music Stream |
|---|---|---|---|
| **Audio Pipeline** | In-process Rodio/Symphonia 32-bit float engine | Spotify Connect device remote control | Official browser embedded player (ADR-003) |
| **Audiophile Codecs** | FLAC, ALAC, WAV, MP3, Vorbis, AAC (up to 192kHz/24-bit) | Ogg Vorbis / AAC (via Spotify Connect device) | AAC / Opus (via official YouTube player) |
| **20Hz–20kHz FFT Spectrum Analyzer** | **Available** (Hann FFT, Stereo RMS/Peak dBFS, Scope) | *Not available* (Informative badge displayed) | *Not available* (Informative badge displayed) |
| **LRCLIB Synchronized Lyrics** | **Available** (automatic offset tracking) | **Available** (synced to remote track metadata) | **Available** (synced to remote track metadata) |
| **Scrubber & Transport Control** | Instantaneous low-latency seek & pause/play | **Available** (1Hz poll + 60 FPS interpolated scrubber) | **Available** (60 FPS interpolated scrubber) |
| **Volume Control** | Bit-perfect in-app Rodio gain & volume leveling | Connect device remote volume | In-app / browser volume |
| **Credentials Storage** | *None required* (100% local-first) | Windows Credential Manager (`keyring` crate) | Windows Credential Manager (`keyring` crate) |
| **Account Requirement** | *None* | Spotify Premium (for Connect device control) | Free Google account (free 10,000 units/day API key) |

> [!NOTE]
> **Why is the Audio Analyzer exclusive to local files?**
> Eclipse's real-time 2048-sample forward FFT spectrum visualizer, stereo peak meters, and LUFS loudness calculation tap directly into the uncompressed in-memory PCM audio stream decoded by Rodio. Spotify Connect and YouTube playback execute through remote endpoints or browser containers where raw PCM frames are not exposed to the local application. For streams, Eclipse elegantly replaces the visualizer with an informative session badge (`STREAM`).

---

### Spotify Connect Setup Guide

Eclipse uses standard **RFC 7636 PKCE (Proof Key for Code Exchange)** for Spotify authentication. This means **no client secret is required**, and credentials never touch plain-text configuration files.

1. **Open Spotify Developer Dashboard**:
   - Go to [developer.spotify.com/dashboard](https://developer.spotify.com/dashboard) and log in with your Spotify account.
2. **Create an Application**:
   - Click **Create App** in the dashboard.
   - Enter an **App name** (e.g. `Eclipse Music Player`) and **App description** (e.g. `Local music player stream integration`).
3. **Configure Redirect URI**:
   - Under **Redirect URIs**, add the exact local loopback callback:
     ```
     http://127.0.0.1:8888/callback
     ```
   - Click **Add** to register the URI.
4. **Select API / SDK**:
   - Under **Which API/SDKs are you planning to use?**, select **Web API**.
   - Accept the Spotify Developer Terms of Service and click **Save**.
5. **Copy Your Client ID**:
   - Navigate to the app's **Settings** tab and copy the **Client ID** (32-character hexadecimal string).
6. **Connect in Eclipse**:
   - Launch Eclipse and click **Spotify** under the `STREAM` section in the left sidebar (or go to `Settings` → `Streaming`).
   - Paste your **Client ID** into the input field and click **Connect with Spotify**.
   - Your default browser will open to Spotify's authorization page. Click **Agree**.
   - Eclipse's background worker intercepts the OAuth code on `127.0.0.1:8888`, retrieves tokens, and securely encrypts them into the **Windows Credential Manager**.
7. **Playing Tracks**:
   - Ensure you have an active Spotify Connect device running (such as the official Spotify desktop client, mobile app, or web player).
   - Search or browse your playlists in Eclipse and click **Play**. Eclipse will automatically stream through your active Spotify Connect device!

---

### YouTube Music Setup Guide

Eclipse uses the official **Google YouTube Data API v3** to search music tracks and playlists, launching official embedded playback in your browser per **ADR-003** with zero scraping or Terms of Service violations.

1. **Open Google Cloud Console**:
   - Visit [console.cloud.google.com](https://console.cloud.google.com/) and sign in with your Google account.
2. **Create or Select a Project**:
   - Click the project dropdown at the top and select **New Project**.
   - Name it (e.g. `Eclipse Music`) and click **Create**.
3. **Enable YouTube Data API v3**:
   - In the navigation menu, select **APIs & Services** → **Library**.
   - Search for `YouTube Data API v3`.
   - Click on the result and click **Enable**.
4. **Generate an API Key**:
   - Navigate to **APIs & Services** → **Credentials**.
   - Click **+ CREATE CREDENTIALS** at the top and choose **API key**.
   - Copy your newly generated API key (starts with `AIzaSy...`).
   - *(Recommended security step)*: Click **Edit API key**, and under **API restrictions**, select **Restrict key** → check **YouTube Data API v3** → click **Save**.
5. **Connect in Eclipse**:
   - In Eclipse, click **YouTube Music** in the sidebar (or go to `Settings` → `Streaming`).
   - Paste your API key into the field and click **Save API Key & Connect**.
   - Eclipse securely encrypts and stores the key in the **Windows Credential Manager**.
6. **Quota & Rate Limits**:
   - Google grants every Cloud Project a free quota of **10,000 units per day**.
   - Eclipse optimizes requests by batch-enriching video durations up to 50 items per call, preserving quota for hours of daily browsing.

---

## ✦ Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Space` | Play / Pause |
| `←` / `→` | Seek ±5 seconds |
| `↑` / `↓` | Volume ±5% |
| `M` | Toggle Mute |
| `S` | Toggle Shuffle |
| `R` | Cycle Repeat Mode (`Off` → `All` → `One`) |
| `Ctrl + K` | Focus Global Search |
| `F11` | Toggle Fullscreen |

---

## ✦ Architecture Overview

```
src/
├── main.rs                 # Application entry point & frameless viewport configuration
├── config.rs               # Persistent user preferences (%APPDATA%\Eclipse\settings.json)
├── audio/
│   ├── engine.rs           # Rodio sink wrapper & non-blocking TapSource PCM interceptor
│   ├── visualizer.rs       # 2048-sample Hann FFT, log dBFS bins, scope & stereo meters
│   └── mod.rs              # PlayerController & queue management
├── library/
│   ├── db.rs               # SQLite library storage & queries (rusqlite)
│   ├── scanner.rs          # Directory walker & audio metadata extraction (lofty)
│   └── model.rs            # Track, Album, Artist data structures
├── lyrics/
│   ├── lrc.rs              # Synced LRC parser with timestamp offsets
│   └── lrclib.rs           # Asynchronous LRCLIB client
└── ui/
    ├── icons.rs            # 100% vector icon component library
    ├── theme.rs            # Design tokens & frosted glass styling frames
    ├── components.rs       # Format badges, buttons, and custom widgets
    ├── playerbar.rs        # 3-column docked bottom player bar & mini visualizer
    ├── sidebar.rs          # 240px navigation sidebar with high-contrast pills
    ├── titlebar.rs         # Custom frameless titlebar with window controls & search
    ├── resize.rs           # Frameless window perimeter hit-testing & resize handling
    ├── art.rs              # Asynchronous cover art loader & texture cache
    ├── fonts.rs            # CJK font fallback configuration
    ├── shortcuts.rs        # Global hotkey event listener
    └── views/              # Home, Now Playing, Songs, Albums, Artists, Playlists, Folders, Settings, About
```

---

## Acknowledgements

Eclipse is an independent Rust reimplementation inspired by [Umbra](https://github.com/sadult/umbra), originally created by **Mersad Shahidi (@Bitologist)**.

The original project inspired the interface design, music-player concept, and overall user experience.

The Rust implementation is independently developed and is not officially affiliated with the original author.

Original project: https://github.com/sadult/umbra

Original licence: MIT License.

---

## ✦ License

Released under the [MIT License](LICENSE).
