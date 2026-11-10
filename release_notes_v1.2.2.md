## ✦ Eclipse v1.2.2 — Zero-Login Spotify Importer & Embedded Streams

Eclipse v1.2.2 replaces proprietary developer-app setups with a sleek, zero-login public Spotify link importer and embedded YouTube music streaming.

### ✨ Highlights & New Features

- **Zero-Login Spotify Link Importer**:
  - Paste any public Spotify track, album, or playlist link (`open.spotify.com/...` or `spotify:...`).
  - Fetches public metadata (titles, artists, albums, duration, artwork, track order) without any developer account, Client ID, or login flow.
  - Automatic fallback through public embed and oEmbed protocols with SQLite caching.

- **Library Matcher ("Find in my library")**:
  - Matches imported stream tracks to local files using CJK-safe Unicode normalization and ±5s duration tolerance.
  - Marks local matches with a `LOCAL` badge and seamlessly plays the bit-perfect lossless file.

- **One-Click Playlist Saver ("Save all as playlist")**:
  - Saves any imported Spotify playlist or album directly into local SQLite playlists with one click.

- **Official Embedded Webview Streaming**:
  - Matches tracks via YouTube Data API v3 (`{artist} - {title}`) with lazy resolution and SQLite match caching.
  - Plays through official embedded YouTube player in a dedicated compact glass window via `wry`/WebView2.
  - Bidirectional IPC controls transport (play/pause, seek, volume, automatic queue advancement on track end).
  - Strictly compliant: zero scrapers, zero `yt-dlp`, zero audio extraction or downloads.

- **100% Vector Iconography**:
  - Pure mathematical vector paths for all icons (including newly added Download icon); zero Unicode emoji dependencies.

### 📦 Assets & Running Standalone
- Download and extract `eclipse-v1.2.2-windows-x64.zip` or run `eclipse.exe` directly.
- Configure your free YouTube Data API v3 key in **Settings > Streaming** to enable YouTube search and stream playback.
