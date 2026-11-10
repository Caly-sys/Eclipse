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

### ADR-003: YouTube Embedded Playback via Dedicated Wry/WebView2 Window (Official IFrame API)
- **Context**: The player bar, Play all, Shuffle, seek, volume, and next/prev controls must directly control YouTube playback, read playback position in real-time, and advance on the "ended" event. `webbrowser::open` does not support bidirectional IPC control.
- **Decision**: Host the official YouTube IFrame Player API inside a controlled, small dedicated webview window using `wry` (backed by Windows WebView2):
  - Uses `https://www.youtube.com/iframe_api` with `enablejsapi=1`.
  - Bidirectional IPC: Eclipse dispatches JavaScript commands (`player.playVideo()`, `player.pauseVideo()`, `player.seekTo(sec)`, `player.setVolume(vol)`), and listens to `onStateChange` events (playing, paused, ended) and time updates.
  - On the "ended" event (state 0), automatically advances to the next track in the queue.
  - Fallback / Question gate: If `wry` encounters compatibility issues with the current egui setup, write `handoffs/QUESTION-WRY.md` and stop per protocol.
- **Status**: Accepted & Supersedes previous browser popout decision.

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
