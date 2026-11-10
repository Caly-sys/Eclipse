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

#### ✦ Build Requirements & Toolchain Prerequisites

Before building Eclipse, ensure your Windows development environment has the following tools installed:

| Requirement | Minimum Version | Installation / Command | Purpose |
|---|---|---|---|
| **Operating System** | Windows 10 (1809+) or Windows 11 (64-bit) | Pre-installed | Target host OS |
| **Rust Toolchain** | `1.80.0+` (`stable-x86_64-pc-windows-msvc`) | `winget install Rustlang.Rustup` or [rustup.rs](https://rustup.rs/) | Rust compiler (`rustc`) & package manager (`cargo`) |
| **Visual Studio C++ Build Tools** | VS 2019 or 2022 (MSVC v142/v143) | `winget install Microsoft.VisualStudio.2022.BuildTools` or [Visual Studio Downloads](https://visualstudio.microsoft.com/visual-cpp-build-tools/) | Provides C++ compiler (`cl.exe`), linker (`link.exe`), and libraries for compiling native C dependencies (`rusqlite`, `cpal`, `wry`) |
| **Windows 10 / 11 SDK** | 10.0.19041.0 or newer | Bundled with VS C++ Workload | Core Windows API headers and libraries (`kernel32`, `user32`, `dwmapi`, `ole32`) |
| **Git for Windows** | 2.40+ | `winget install Git.Git` or [git-scm.com](https://git-scm.com/) | Version control and repository cloning |
| **Python 3.8+** *(Optional)* | 3.8+ | `winget install Python.Python.3.12` | Required only if running the emoji audit test script (`tests/verify_no_emojis.py`) |

---

#### ✦ Installing the C++ Build Environment (Quick Setup)

If you do not have Visual Studio installed, you can install the exact required C++ build tools in one command using `winget`:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Alternatively, open the **Visual Studio Installer** and verify the following components are checked:
- ✅ **Desktop development with C++** (Workload)
  - ✅ **MSVC v143 - VS 2022 C++ x64/x86 build tools**
  - ✅ **Windows 11 SDK (10.0.22621.0)** or **Windows 10 SDK**

Then verify your Rust toolchain uses the MSVC target:
```powershell
rustup default stable-x86_64-pc-windows-msvc
```

---

#### ✦ Clone & Compile

```powershell
# 1. Clone the repository
git clone https://github.com/Caly-sys/Eclipse.git
cd Eclipse

# 2. Build for development (fast debug build)
cargo build

# 3. Build optimized standalone release binary
# (Automatically statically links the C-runtime via .cargo/config.toml)
cargo build --release

# 4. Launch Eclipse
.\target\release\eclipse.exe
# or use the bundled quick-launch script:
.\run.bat
```

---

#### ✦ Running Tests & Code Quality Gates

Eclipse enforces a 100% clean test suite with zero compiler warnings and zero Unicode emoji artifacts:

```powershell
# Run the complete test suite (156 unit & integration tests)
cargo test --all

# Run the strict Clippy linter (zero warnings policy)
cargo clippy --all-targets -- -D warnings

# Run the zero-emoji verification audit (requires Python)
python tests/verify_no_emojis.py
```

---

#### ✦ Packaging the Portable Distribution ZIP

To create the portable standalone distribution package:

```powershell
# 1. Ensure the release binary is built
cargo build --release

# 2. Copy the binary to the root directory
Copy-Item target\release\eclipse.exe eclipse.exe -Force

# 3. Create the release archive
Compress-Archive -Path eclipse.exe, run.bat, README.md, LICENSE -DestinationPath eclipse-windows-x64.zip -Force
```

---

#### ✦ Troubleshooting Build Issues

- **`error: linker 'link.exe' not found`**:
  You are missing the Microsoft C++ Build Tools. Install the **Desktop development with C++** workload in Visual Studio Installer and restart your terminal.
- **`VCRUNTIME140.dll` missing on other PCs**:
  Ensure you build with `.cargo/config.toml` containing `rustflags = ["-C", "target-feature=+crt-static"]` (already included in the repo). This statically embeds the C-runtime into the executable.
- **Slow initial compile**:
  The first `cargo build` compiles native audio codecs and GUI dependencies (~3–4 minutes). Subsequent incremental builds take 2–5 seconds.

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

## ✦ Streaming & Spotify Importer

Starting in **v1.2.2**, Eclipse includes a powerful, zero-login **Spotify Public Link Importer** and integrated **YouTube Music** streaming playback alongside your local lossless library.

### How It Works

1. **Paste Any Public Spotify Link**:
   - Copy any public Spotify track, album, or playlist link (e.g. `https://open.spotify.com/playlist/...`, `/album/...`, `/track/...` or `spotify:track:...`).
   - Paste it directly into the Spotify importer bar in Eclipse.
2. **Instant Zero-Login Metadata Extraction**:
   - Eclipse parses the link and fetches track titles, artists, album names, durations, and high-resolution artwork directly from public embeds and oEmbed metadata.
   - **Zero Login & Zero Developer Account**: No Spotify account, no Spotify Developer app, no Client ID, no Client Secret, and no redirect URIs are required.
3. **Smart Local Library Matcher**:
   - Click **Find in my library**: Eclipse automatically compares the imported track titles and artists against your indexed local music files (`.flac`, `.wav`, `.mp3`).
   - Uses intelligent fuzzy matching with CJK/Unicode normalization, punctuation stripping, and duration tolerance (±4 seconds).
   - If an imported track already exists in your local library, Eclipse marks it with a bright `LOCAL` badge and will play your pristine bit-perfect lossless file instead of streaming!
4. **Save As Local Playlist**:
   - Click **Save all as playlist**: Eclipse instantly saves the imported tracklist into your persistent SQLite library database as a custom Eclipse playlist.
5. **Matched YouTube Streaming Playback**:
   - For tracks not found in your local collection, Eclipse matches them to the official YouTube Music / YouTube video catalog and plays them via the official embedded player without scrapers or third-party TOS violations.

---

### Capabilities Matrix: What Streams Can and Can't Do

| Feature / Capability | Local Lossless Playback | Spotify Public Link Import | YouTube Music Stream |
|---|---|---|---|
| **Audio Pipeline** | In-process Rodio/Symphonia 32-bit float engine | Metadata import + local match / YouTube stream | Official embedded webview player (ADR-003) |
| **Audiophile Codecs** | FLAC, ALAC, WAV, MP3, Vorbis, AAC (up to 192kHz/24-bit) | Plays local lossless file if matched | AAC / Opus (via official embedded player) |
| **20Hz–20kHz FFT Spectrum Analyzer** | **Available** (Hann FFT, Stereo RMS/Peak dBFS, Scope) | **Available** when playing local matched file | *Not available for remote streams* (badge displayed) |
| **LRCLIB Synchronized Lyrics** | **Available** (automatic offset tracking) | **Available** (synced to track title & artist) | **Available** (synced to track title & artist) |
| **Save as Custom Playlist** | **Available** (instant SQLite persistence) | **Available** (one-click "Save all as playlist") | **Available** |
| **Account / Login Requirement** | **None** (100% local-first) | **None** (Zero-login public embed parser) | Free Google account (for YouTube API key) |
| **Credentials Storage** | *None required* | *None required* | Windows Credential Manager (`keyring` crate) |

> [!NOTE]
> **Why is the Audio Analyzer exclusive to local files?**
> Eclipse's real-time 2048-sample forward FFT spectrum visualizer, stereo peak meters, and LUFS loudness calculation tap directly into the uncompressed in-memory PCM audio stream decoded by Rodio. Remote streaming playback executes through official player containers where raw PCM frames are not exposed to the local application. For streams, Eclipse displays an informative session badge (`STREAM`).

---

### YouTube Music API Key Setup Guide

Eclipse uses the official **Google YouTube Data API v3** to search music tracks and match imported Spotify songs.

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
5. **Configure in Eclipse**:
   - In Eclipse, click **Settings** (`⚙ Settings` in sidebar) → **Streaming** tab (or open **YouTube Music** in the sidebar).
   - Paste your API key into the field and click **Save Key & Connect**.
   - Eclipse securely encrypts and stores the key in the **Windows Credential Manager** (`CredWriteW`).
6. **Quota & Rate Limits**:
   - Google grants every Cloud Project a free quota of **10,000 units per day**.
   - Eclipse caches search matches in SQLite (`stream_match_cache`) to avoid repeated queries and preserve your quota.

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
