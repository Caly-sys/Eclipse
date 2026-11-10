# Architecture Decisions Record (ADR) - Streams

## Decision Log

### ADR-001: Separation of Playback Engines via `PlaybackSource`
- **Context**: Eclipse uses a high-performance, low-latency Rodio/Symphonia local audio engine for FLAC, WAV, MP3 lossless decoding with FFT spectrum analysis. Streaming services (Spotify, YouTube) operate either via remote device control (Spotify Connect) or official browser/embedded sessions.
- **Decision**: Introduce `PlaybackSource` enum (`Local | Stream(String)`) in `src/streams/provider.rs` and integrate it into `PlayerController` (`src/audio/mod.rs`). Local playback remains 100% unchanged and continues to use the existing Rodio pipeline. All local lossless decoding logic, gapless transitions, equalizer, and volume leveling continue uninterrupted.
- **Status**: Accepted & Implemented (TASK-S1).

### ADR-002: Zero-Secret Principle & Windows Credential Manager
- **Context**: Users authenticate with their own Spotify Client ID or YouTube API Key. Spotify OAuth yields sensitive refresh tokens and access tokens. Under no circumstances should secrets leak into `settings.json`, SQLite databases, Git history, terminal stdout/stderr, or UI logs.
- **Decision**: Store all sensitive tokens in the Windows Credential Manager using the `keyring` crate with `features = ["windows-native"]`. Implemented in `src/streams/credentials.rs` (`CredentialStore`). Service namespaces `Eclipse_Spotify` and `Eclipse_YouTube` isolate credentials with zero plain-text logging. Missing credentials safely return `Ok(None)`.
- **Status**: Accepted & Implemented (TASK-S1).

### ADR-003: YouTube Playback Hosting Strategy
- **Context**: User requirement: *"Record decisions in DECISIONS.md, including how YouTube playback is hosted (options: a separate small webview window with the official embedded player, or opening the default browser). Pick the simplest that works."*
- **Evaluation**:
  - *Option A (Heavy embedded webview via wry/tao)*: Introduces massive C++/Edge WebView2 native build dependencies, potential CI compilation failures on Windows MSVC, huge binary size inflation, and complicated IPC event bridges.
  - *Option B (Lightweight official webview / popup window)*: Uses Windows native Shell/Edge popup URL to host the official YouTube embed player.
  - *Option C (Opening the default browser at the official embedded / watch URL)*: 100% stable, zero additional C++ runtime dependencies, zero binary bloat, no DLL dependencies, zero risk of broken builds, and fully compliant with YouTube's Terms of Service and official monetization/player guidelines.
- **Decision**: Select **Option C (official embedded / watch player launched in user's default browser via `webbrowser` crate off-thread)** as the primary simplest working solution. Eclipse provides search, playlist browsing, track listing, and one-click playback launch.
- **Status**: Accepted & Implemented (TASK-S1).

### ADR-004: Pure Vector Icons for Streaming Providers
- **Context**: Eclipse has a strict "Zero Emojis" vector icon policy.
- **Decision**: Render Spotify and YouTube icons using crisp mathematical vector paths in `src/ui/icons.rs` (`Icon::Spotify`, `Icon::YouTube`).
- **Status**: Accepted.

### ADR-005: Non-Blocking Asynchronous Channel Architecture
- **Context**: All network I/O (OAuth loops, Spotify Web API, YouTube Data API, HTTP requests, LRCLIB lyrics) must remain strictly off the egui 60 FPS UI rendering thread.
- **Decision**: Implement a bi-directional actor channel architecture via `std::sync::mpsc`:
  - `StreamCommand`: Dispatched from UI / PlayerController to background worker (`eclipse-stream-worker`).
  - `StreamEvent`: Non-blocking stream events drained per-frame by `StreamController::update()`.
  - Providers execute their heavy network calls synchronously on the background worker thread, ensuring the UI thread never experiences frame stutter or micro-freezes.
- **Status**: Accepted & Implemented (TASK-S1).

### ADR-006: Automatic Mutual Audio Exclusivity
- **Context**: Local audio and streaming audio should never play simultaneously over each other.
- **Decision**:
  - Calling `player.switch_to_stream(provider, track)` automatically pauses the local Rodio engine.
  - Calling `player.load_queue(...)` or `player.play_index(...)` automatically resets `player.playback_source` to `PlaybackSource::Local` and clears `active_stream_track`.
  - While in `PlaybackSource::Stream`, local `play_pause` / `toggle_playback` will not inadvertently resume paused local audio tracks.
- **Status**: Accepted & Implemented (TASK-S1).
