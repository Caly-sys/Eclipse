## ✦ Eclipse v1.2.3 — Spotify-Style Two-Tier Queue & Track Context Menu

Eclipse v1.2.3 introduces a Spotify-style two-tier playback queue with index permutation shuffle and smart artist spread, alongside a floating dark monochrome Track Context Menu with cascading "Add to playlist" submenu and pixel-perfect viewport edge detection.

### ✦ Highlights & Fixes

- **Spotify-Style Track Context Menu & "Add to Playlist" Submenu**:
  - Right-click any track row or click the ••• action button on hover to open a floating dark monochrome context menu rendered as a top-level overlay (`egui::Area` on `Order::Foreground`).
  - Zero clipping inside scroll containers with smart viewport edge detection (flips upward above bottom playerbar, cascades submenu to the left near screen edges).
  - Cascading **"Add to playlist ▸"** submenu with instant search filtering, inline **"+ New playlist"** creation, and scrollable playlist list.
  - Contextual **"Remove from this playlist"** action (only visible when viewing a playlist).
  - **"Play next"** and **"Add to queue"** integration wired directly into the two-tier playback queue.
  - **"Add to Liked Songs"** / **"Remove from Liked Songs"**, **"Go to artist"**, **"Go to album"**, and **"Show in file explorer"**.
- **Spotify-Style Two-Tier Playback Queue & Smart Shuffle**:
  - Implemented true two-tier queue resolution: user-queued tracks ("Play Next" / "Add to Queue") drain completely before linear or shuffled playlist progression resumes.
  - Playlist progression pointer (`queue_pos`) is preserved while draining user queue, seamlessly picking up where the playlist left off.
  - Index permutation shuffle with active-track pinning at index 0 and Smart Artist Spread pass (minimizes back-to-back clustering of songs by the same artist).
  - Toggling shuffle OFF smoothly resumes linear order from current track without index jumps.
  - Bounded 100-track backward navigation history stack for accurate previous track traversal.
- **Playerbar Transport Buttons Alignment Fix**:
  - Resolved egui row-height cursor expansion bug where Shuffle and Previous (|◀) were top-aligned at 21px while Pause and Next (▶|) sat lower at 40px.
  - Pre-allocated 40.0px row interaction height, guaranteeing that Shuffle, Previous, Play/Pause, Next, and Repeat are centered on the exact same horizontal centerline.
  - Applied matching alignment stabilization to PiP overlay transport controls.
- **Audio Engine Seeking & FLAC Decoding Fixes**:
  - Implemented `SymphoniaDecoderSource` with exact media byte lengths, enabling native binary seek tables across FLAC, MP3, WAV, AAC, and OGG formats.
  - Resolved Rodio empty sink EOF recovery: seeking on completed tracks seamlessly restarts audio.
  - Synchronized playback progress directly with real hardware sample clocks (`sink.get_pos()`), eliminating timer drift and frozen bars.
- **Seekbar Scrubbing & Window Edge Protection**:
  - Excluded the playerbar center from window edge `ResizeSouth` handling to eliminate mouse capture conflicts.
  - Implemented visual scrub preview with single-commit on drag release and anti-snapback knob hold.
  - Unified seeking across playerbar, keyboard arrow shortcuts, synced lyrics in Now Playing, and PiP mini seekbar.
- **Spotify Public Link Importer & Webview Streaming**:
  - Paste any public Spotify track, album, or playlist link without developer credentials or login.
  - Matches imported stream tracks against local library using CJK-safe Unicode normalization.
  - Official embedded YouTube player streaming with bidirectional transport IPC.

### ✦ SHA-256 Checksums (v1.2.3)

| File | SHA-256 Checksum |
|---|---|
| `eclipse.exe` | `03BFC942025395D889C98673CD41EA92ACE5FC19D9EFB0F46366DDD3072D43E2` |
| `eclipse-v1.2.3-windows-x64.zip` | `CA3F62088AF91EEBBC6DFF43D0D9F93DF7689A49780643057DCFC44D9742C02C` |

### ✦ Installation
- Download and extract `eclipse-v1.2.3-windows-x64.zip` or run `eclipse.exe` directly.
- Configure your free YouTube Data API v3 key in **Settings > Streaming** to enable YouTube search and stream playback.
