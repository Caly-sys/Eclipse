# Eclipse Streams Rebuild Task Board

| Task ID | Description | Assignee | Status | Deliverable |
|---|---|---|---|---|
| **TASK-R1** | Delete legacy PKCE, 127.0.0.1 redirect, Connect API, clean up dead tests & settings | `cleanup-agent` | `HANDOFF_READY` | Clean build without developer app/Connect code, `handoffs/TASK-R1.md` |
| **TASK-R2** | Public Spotify link parser & embed/oEmbed metadata fetcher (no login) | `link-import-agent` | `HANDOFF_READY` | `src/streams/spotify_public.rs`, `handoffs/TASK-R2.md` |
| **TASK-R3** | "Save as playlist" & "Find in my library" matcher with CJK/fuzzy normalization | `library-match-agent` | `HANDOFF_READY` | `src/streams/matcher.rs`, `handoffs/TASK-R3.md` |
| **TASK-R4** | YouTube Data API v3 matcher & official embedded playback with match cache | `playback-agent` | `TODO` | `src/streams/playback.rs`, `handoffs/TASK-R4.md` |
| **TASK-R5** | Rebuilt Spotify Importer UI matching reference image, YouTube search, and playerbar | `stream-ui-agent` | `TODO` | `src/ui/views/streams.rs`, `src/ui/sidebar.rs`, `handoffs/TASK-R5.md` |
| **TASK-R6** | End-to-end regression audit, upgrade migration test, 102+ test pass gate, documentation | `qa-reviewer` | `TODO` | `docs/streams/BUGS.md`, `README.md`, `handoffs/TASK-R6.md` |
