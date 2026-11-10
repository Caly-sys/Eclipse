# Eclipse Seekbar & Seeking Engine Root Cause Analysis (RCA)

**Project:** Eclipse (C:\music\Eclipse)  
**Investigation Date:** November 2026  
**Investigator:** seek-investigator  
**Target:** Diagnose root causes of seekbar failure ("seekbar in bottom player bar does not seek; clicking or dragging does nothing; find related seeking bugs").

---

## Executive Summary

The seekbar failure is caused by a compound multi-layer architectural breakdown spanning UI hit-testing, native window edge collision, decoding library limitations, and state synchronization:

1. **Native Window Border Hijacking (Area 2 - Critical UI Blocker):**
   In floating (non-maximized) window mode, the bottom docked playerbar allocates 68px height, but its vertical contents (margins + 40px play button + 3px gap + 18px seekbar hit rect) total 69px. This pushes the bottom half of the seekbar (`[screen_rect.max.y - 7.0, screen_rect.max.y + 1.0]`) directly into `RESIZE_MARGIN = 7.0px`. `src/ui/resize.rs` executes before all UI panels and unconditionally detects `ResizeDirection::South`, dispatching `ViewportCommand::BeginResize(South)`. Native Windows OS resizing captures mouse pointer events before the seekbar ever registers clicks or drags.
2. **Rodio Decoder Unseekable Error (Area 3 - Critical Engine Blocker):**
   Even when clicks/drags bypass resize collision (e.g. In maximized mode or top pixels), **audio seeking in local lossless audio files completely fails**. Rodio 0.19's `Decoder::new` wraps files in an internal `ReadSeekSource` struct whose `MediaSource::byte_len()` is hardcoded to return `None`. Symphonia's FLAC demuxer (`symphonia-bundle-flac-0.5.5\src\demuxer.rs:235`) explicitly requires `self.reader.byte_len().ok_or(Error::SeekError(SeekErrorKind::Unseekable))?`. Thus, `sink.try_seek(pos)` fails with `Err(SymphoniaDecoder(BaseSeek(SeekError(Unseekable))))`. The error is silently discarded with `let _ = sink.try_seek(pos);` in `src/audio/engine.rs:764`.
3. **Wall-Clock Position Timer Desynchronization & Frozen Bar (Area 4):**
   `AudioEngine::position()` does not query hardware or decoder sample clocks; it computes `(self.pause_offset + start.elapsed()).min(self.track_duration)`. When `seek()` is called, `self.pause_offset` is updated while the audio sink remains stuck at the old position. This creates complete desync between UI and audio. When the fake timer hits `track_duration`, the seekbar clamps and **freezes at 100%** while audio continues playing for minutes, or prematurely triggers gapless track transition.
4. **Drag Flooding and Drag-End Snapback (Area 1):**
   Dragging the seekbar calls `seek()` on every single frame (up to 60 times/sec). For streams, this spams webview IPC. On mouse release, egui reports `is_dragged = false` and `resp.clicked() = false`, causing `display_pos` to snap back to the stale engine position.
5. **Secondary Seek Entry Points Broken (Area 5):**
   Left/Right arrow keys (`shortcuts.rs`) crash stream seeking because `player.current_track()` is `None` (duration defaults to 0). PiP overlay seekbar (`pip.rs`) is completely broken for streams and fails on FLAC.

---

## Area 1: Input Handling & Hit Testing

### Citations:
- `src/ui/playerbar.rs:375-400`
- `src/ui/playerbar.rs:531-607`

### 1.1 Hit Rect Allocation & Geometry
In `src/ui/playerbar.rs:540`:
```rust
let (rect, resp) = ui.allocate_exact_size(vec2(desired_width, 18.0), egui::Sense::click_and_drag());
```
- Rect size: `desired_width` (derived from `(center_w - 90.0).max(60.0)`) by `18.0px`.
- Sense: Correctly allocated with `egui::Sense::click_and_drag()`.
- Visual bar:
  ```rust
  let bar_height = if is_hovered || is_dragged { 4.5 } else { 3.0 };
  let bar_rect = Rect::from_center_size(rect.center(), vec2(rect.width(), bar_height));
  ```
  The interactive hit target is 18.0px tall, while the visual line is 3.0px (4.5px on hover).

### 1.2 Target Ratio & Math
```rust
let ratio = ((pointer_pos.x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);
let target_secs = ratio as f64 * max_dur;
```
The horizontal ratio math is correct. When `interact_pointer_pos()` is available, `ratio` maps linearly from 0.0 to 1.0.

### 1.3 Per-Frame Drag Flooding vs Release Behavior
In lines 555-570:
```rust
let display_pos = if is_dragged || resp.clicked() {
    if let Some(pointer_pos) = resp.interact_pointer_pos() {
        ...
        if let Some((prov_id, stream_ctrl)) = stream_info {
            stream_ctrl.seek(prov_id, target_secs);
        } else {
            player.engine.seek(Duration::from_secs_f64(target_secs));
        }
        target_secs
    } else {
        pos
    }
} else {
    pos
};
```
- **Every Frame Scrubbing:** `is_dragged` evaluates to `true` on every frame while the mouse moves with the button down. This causes `player.engine.seek(...)` or `stream_ctrl.seek(...)` to be dispatched up to 60 times per second.
  - On streaming tracks, this floods webview IPC with `PlayerWebviewCommand::Seek`, causing YouTube embed player command drops and stutter.
  - On local tracks, it repeatedly resets `start_instant` and clears `AudioTapBuffer` 60 times/sec.
- **Drag Release Snap-Back:** When the user releases the mouse button after dragging:
  - In egui, `resp.dragged()` transitions from `true` to `false`.
  - `resp.clicked()` is `false` because the pointer exceeded the drag click tolerance.
  - On that release frame, `is_dragged || resp.clicked()` is `false`.
  - `display_pos` falls back to `pos`. If the audio engine or webview has not yet updated its playback timestamp, the seekbar knob snaps back to the previous position before lurching forward.

---

## Area 2: Layout & Window Edge Resize Collision

### Citations:
- `src/ui/resize.rs:3-68`
- `src/ui/mod.rs:300`
- `src/ui/mod.rs:387-393`
- `src/ui/mod.rs:826-842`
- `src/ui/playerbar.rs:24-26`
- `src/ui/playerbar.rs:249-254`

### 2.1 The Vertical Dimension Stack Calculation
When the window is floating (not maximized, not fullscreen):
- Window screen rect: `[0.0, 0.0] - [W, H]`.
- Top titlebar + margins: ~42px.
- Main workspace height: `workspace_height = (ui.available_height() - playerbar_h).max(100.0)`.
- Docked playerbar allocated height: `playerbar_h = 68.0px`.
  - Docked playerbar outer rect Y range: `[H - 68.0, H]`.
- Playerbar frame inner margin (`src/ui/playerbar.rs:26`): `Margin::symmetric(20.0, 8.0)`.
  - Top inner padding: 8.0px.
  - Available inner Y starts at: `H - 68.0 + 8.0 = H - 60.0`.
- Center column layout (`src/ui/playerbar.rs:249`):
  - Row 1: Transport buttons (Play/Pause button: 40px high). Y range: `[H - 60.0, H - 20.0]`.
  - Item spacing (`spacing_mut().item_spacing = vec2(0.0, 3.0)`): 3.0px.
  - Row 2: Horizontal Seekbar row (`allocate_exact_size(vec2(w, 18.0))`):
    - Row 2 Y starts at: `H - 20.0 + 3.0 = H - 17.0`.
    - Row 2 Y ends at: `H - 17.0 + 18.0 = H + 1.0`!
  - Seekbar vertical center: `(H - 17.0 + H + 1.0) / 2.0 = H - 8.0`.

### 2.2 Collision with Window Resize Border
In `src/ui/resize.rs:4`:
```rust
pub const RESIZE_MARGIN: f32 = 7.0;
```
In `detect_resize_direction` (`src/ui/resize.rs:26`):
```rust
let on_bottom = pos.y >= screen_rect.max.y - margin; // pos.y >= H - 7.0
```
In `handle_window_resize` (`src/ui/resize.rs:60-65`):
```rust
if let Some((direction, cursor)) = detect_resize_direction(screen_rect, pos, RESIZE_MARGIN) {
    ctx.set_cursor_icon(cursor);

    if ctx.input(|i| i.pointer.primary_down() && (i.pointer.is_decidedly_dragging() || i.pointer.primary_pressed())) {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}
```
### Evidence of Collision:
1. `handle_window_resize(ctx)` is called in `src/ui/mod.rs:300` **before** any widget or CentralPanel rendering.
2. It does not check `ctx.wants_pointer_input()` or `ctx.is_pointer_over_area()`.
3. The lower 8 pixels of the seekbar hit rect lie between `H - 7.0` and `H + 1.0`.
4. When a user hovers over or clicks the lower half of the seekbar or drags downwards slightly, `detect_resize_direction` returns `Some((ResizeDirection::South, CursorIcon::ResizeSouth))`.
5. `handle_window_resize` immediately sends `ViewportCommand::BeginResize(ResizeDirection::South)` to the Windows operating system.
6. The OS initiates a window resize grab, consuming all subsequent pointer events. Egui never delivers the click or drag to the seekbar.

---

## Area 3: Audio Engine & Decoder Seeking Failure

### Citations:
- `src/audio/engine.rs:104-110` (TapSource try_seek)
- `src/audio/engine.rs:361` (restore_playback try_seek)
- `src/audio/engine.rs:759-774` (AudioEngine::seek)
- `rodio-0.19.0\src\source\samples_converter.rs:44-47`
- `rodio-0.19.0\src\decoder\mod.rs:167-183, 228-238`
- `rodio-0.19.0\src\decoder\read_seek_source.rs:17-25`
- `symphonia-bundle-flac-0.5.5\src\demuxer.rs:233-236`
- `symphonia-bundle-mp3-0.5.5\src\demuxer.rs:397-400`
- `symphonia-format-isomp4-0.5.5\src\demuxer.rs:340`

### 3.1 Trace of UI Event to Audio Device
```
User Click/Drag
   │
   ▼
src/ui/playerbar.rs:562: player.engine.seek(Duration::from_secs_f64(target_secs))
   │
   ▼
src/audio/engine.rs:764: sink.try_seek(pos)
   │
   ▼
rodio::Sink::try_seek(pos)
   │ (sends SeekOrder to rodio audio mixer thread)
   ▼
src/audio/engine.rs:104: TapSource::try_seek(pos)
   │
   ▼
rodio::source::SamplesConverter::try_seek(pos)
   │
   ▼
rodio::Decoder::try_seek(pos)
   │
   ▼
rodio::decoder::symphonia::SymphoniaDecoder::try_seek(pos)
   │
   ▼
symphonia::core::formats::FormatReader::seek(SeekMode::Accurate, SeekTo::Time { time, .. })
   │
   ▼
symphonia_bundle_flac::demuxer::FlacReader::seek()
   │
   ❌ FAILS! Returns Err(SeekError(SeekErrorKind::Unseekable))
```

### 3.2 The Root Cause of Rodio Unseekable Failure
In `rodio-0.19.0\src\decoder\mod.rs:228-238`:
```rust
let mss = MediaSourceStream::new(
    Box::new(ReadSeekSource::new(data)) as Box<dyn MediaSource>,
    Default::default(),
);
match symphonia::SymphoniaDecoder::new(mss, None) {
    ...
}
```
In `rodio-0.19.0\src\decoder\read_seek_source.rs:17-25`:
```rust
impl<T: Read + Seek + Send + Sync> MediaSource for ReadSeekSource<T> {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        None // <--- HARDCODED TO NONE IN RODIO!
    }
}
```
In `symphonia-bundle-flac-0.5.5\src\demuxer.rs:233-236`:
```rust
let mut start_byte_offset = self.first_frame_offset;
let mut end_byte_offset =
    self.reader.byte_len().ok_or(Error::SeekError(SeekErrorKind::Unseekable))?;
```
In `symphonia-format-isomp4-0.5.5\src\demuxer.rs:340`:
```rust
let len = mss.byte_len().ok_or(Error::SeekError(SeekErrorKind::Unseekable))?;
```
In `symphonia-bundle-mp3-0.5.5\src\demuxer.rs:397-400`:
MP3s without Xing TOC require `self.reader.byte_len()` to compute seek byte offsets.

### 3.3 Concrete Test Probe Results
A probe test was executed directly on `C:\music\Muse\Aimyon - Marigold.flac`:
```
Decoder Channels: 2, Sample rate: 48000, Total duration: Some(306.000000001s)
Direct decoder.try_seek(30s): Err(SymphoniaDecoder(BaseSeek(SeekError(Unseekable))))
convert_samples::<f32>().try_seek(30s): Err(SymphoniaDecoder(BaseSeek(SeekError(Unseekable))))
sink.try_seek(30s) on SamplesConverter(Decoder): Err(SymphoniaDecoder(BaseSeek(SeekError(Unseekable))))
```
**Conclusion:** `sink.try_seek` in rodio with Symphonia feature **always returns `SeekError(Unseekable)` for FLAC files** and any format requiring stream length for seeking.
And in `src/audio/engine.rs:764`:
```rust
let _ = sink.try_seek(pos);
```
The error is discarded. The audio sink continues playing from the original position, completely ignoring the user's seek command.

---

## Area 4: State Management & Desynchronization

### Citations:
- `src/audio/engine.rs:487-493` (is_approaching_eof)
- `src/audio/engine.rs:518-527` (has_track_reached_end)
- `src/audio/engine.rs:539-557` (update_crossfade)
- `src/audio/engine.rs:759-774` (AudioEngine::seek)
- `src/audio/engine.rs:776-785` (AudioEngine::position)
- `src/audio/mod.rs:501-528` (PlayerController::update)

### 4.1 Position Is Pure Wall-Clock Time
In `src/audio/engine.rs:776-785`:
```rust
pub fn position(&self) -> Duration {
    if self.is_paused {
        self.pause_offset
    } else if let Some(start) = self.start_instant {
        let elapsed = start.elapsed();
        (self.pause_offset + elapsed).min(self.track_duration)
    } else {
        self.pause_offset
    }
}
```
`position()` is calculated solely from wall-clock time (`start.elapsed() + self.pause_offset`). It has zero coupling to actual audio samples rendered by CPAL or decoded by Symphonia.

### 4.2 Desync Cycle & The Frozen Bar Bug
When `seek(pos)` is called:
1. `self.pause_offset = pos;`
2. `self.start_instant = Some(Instant::now());`
3. `sink.try_seek(pos)` fails silently.
4. Position leaps to `pos` in the UI.
5. The audio hardware continues playing from the original time (e.g. 0:05).
6. As time passes, the wall-clock position reaches `track_duration`.
7. `(self.pause_offset + elapsed).min(self.track_duration)` clamps to `track_duration`.
8. `is_finished()` checks `sink.empty()`. Since the sink is still playing from the original position (e.g. 2:00 out of 4:00), `sink.empty()` is `false`.
9. **The seekbar freezes at 100% (the end of the track) while audio continues playing for another 2 minutes.**

### 4.3 Premature Gapless Track Transition Bug
In `src/audio/mod.rs:501-528`:
If the user seeks forward near the end of a track:
1. `is_approaching_eof(1500ms)` checks `self.track_duration.saturating_sub(pos) <= 1500ms`.
2. Because the fake wall-clock position is near the end, `prequeue_file` pre-queues track 2 into the sink.
3. 1.5 seconds later, `has_track_reached_end()` returns `true`.
4. `PlayerController::update` calls `transition_to_prequeued()`.
5. The UI switches to Track 2, loads Track 2 lyrics, and updates title and album art.
6. **Track 1 is still playing in the audio sink.** The user hears Track 1 while looking at Track 2.

### 4.4 Prequeue Retention on Backward Seek
If track 2 was already prequeued, and the user seeks backwards:
1. `self.prequeued` remains `true` (it is never reset in `AudioEngine::seek`).
2. `sink` still contains track 2 appended at the tail of its queue.
3. When track 1 eventually finishes in the decoder, track 2 will immediately start playing, completely out of sync with user navigation.

### 4.5 Crossfade Attenuation Lockup
In `src/audio/engine.rs:760-762`:
```rust
if let Some((old_sink, _, _)) = self.crossfade_sink.take() {
    old_sink.stop();
}
```
If a crossfade was active when the user seeks:
- In `play_file()`, `new_sink` was initialized with `volume = 0.0` (line 634).
- `AudioEngine::seek` stops `old_sink` and sets `self.crossfade_sink = None`.
- **`AudioEngine::seek` never resets `self.sink`'s volume back to `self.volume`.**
- Since `self.crossfade_sink` is now `None`, `update_crossfade()` no longer runs.
- **Playback becomes completely muted or stuck at near-zero volume until the user touches the volume slider.**

### 4.6 Paused State Seek Behavior
If paused:
- `self.pause_offset = pos;`
- `self.start_instant` remains `None`.
- `sink.try_seek(pos)` fails.
- When `resume()` is clicked, `sink.play()` resumes from the paused sample, while `self.start_instant = Some(Instant::now())` begins counting forward from `pos`. UI and audio are desynchronized immediately upon resume.

### 4.7 AudioTapBuffer vs TapSource::chunk Stale Samples
In `AudioEngine::seek`:
```rust
if let Ok(mut tap) = self.tap_buffer.lock() {
    tap.clear();
}
```
The tap ring buffer is cleared. However, `TapSource::chunk` has up to 256 unpushed samples. In `TapSource::try_seek`:
```rust
let res = self.input.try_seek(pos);
if res.is_ok() {
    self.chunk.clear();
}
res
```
Since `self.input.try_seek` returns `Err`, `self.chunk.clear()` is skipped. The stale samples are pushed into the visualizer on the very next frame.

---

## Area 5: Secondary Seek Entry Points

### 5.1 Left / Right Arrow Keys (Shortcuts)
- **File:** `src/ui/shortcuts.rs:47-66`
- **Code:**
  ```rust
  if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
      let cur = player.engine.position().as_secs_f64();
      let new_pos = (cur - 5.0).max(0.0);
      player.engine.seek(Duration::from_secs_f64(new_pos));
      ...
  }
  if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
      let cur = player.engine.position().as_secs_f64();
      let dur = player.current_track().map(|t| t.duration_secs).unwrap_or(0.0);
      let new_pos = (cur + 5.0).min(dur);
      player.engine.seek(Duration::from_secs_f64(new_pos));
      ...
  }
  ```
- **Bug 1 (Streaming Broken):** When streaming, `player.playback_source.is_stream()` is true and `player.current_track()` is `None`. For Right Arrow, `dur` is `0.0`. `(cur + 5.0).min(0.0)` produces `new_pos = 0.0`. Left and Right arrows call `player.engine.seek` instead of `stream_controller.seek(...)`, rendering arrow seeking completely non-functional during streaming.
- **Bug 2 (FLAC Broken):** For local tracks, calls `player.engine.seek()`, triggering the unseekable FLAC failure.

### 5.2 Synced Lyrics Click-to-Seek
- **File:** `src/ui/views/now_playing.rs:604` (Local) & `src/ui/views/now_playing.rs:1395` (Stream)
- **Local:** `player.engine.seek(Duration::from_millis(line.timestamp_ms));`
  Fails on all FLAC files due to rodio unseekable bug. The lyric line highlights, but audio continues uninterrupted from the previous location.
- **Stream:** `sc.seek(&track.provider, seek_secs);` correctly dispatches to `StreamController`.

### 5.3 PiP (Picture-in-Picture) Mini Seekbar
- **File:** `src/ui/pip.rs:400-426`
- **Code:**
  ```rust
  let pos_secs = player.engine.position().as_secs_f64();
  let dur_secs = player
      .current_track()
      .map(|t| t.duration_secs)
      .unwrap_or(0.0)
      .max(0.001);
  ...
  if resp.clicked() || resp.dragged() {
      if let Some(pointer_pos) = resp.interact_pointer_pos() {
          let frac = ((pointer_pos.x - rail_rect.min.x) / rail_rect.width()).clamp(0.0, 1.0);
          let seek_target = frac as f64 * dur_secs;
          current_pos = seek_target;
          player.engine.seek(Duration::from_secs_f64(seek_target));
      }
  }
  ```
- **Bug 1 (Streaming Dead):** `dur_secs` defaults to `0.001` and `pos_secs` is `0.0` on streams because `player.current_track()` is `None` and `player.engine` is idle. Seeking calls `player.engine.seek` instead of `stream_controller.seek`.
- **Bug 2 (FLAC Broken):** Local seek calls `player.engine.seek`, failing on FLAC.

### 5.4 Streamed YouTube Playback Seeking
- **File:** `src/streams/controller.rs:890-901`
- **File:** `src/streams/playback.rs:790-793`
- **Code:**
  ```rust
  PlayerWebviewCommand::Seek(pos) => {
      format!("if (player && typeof player.seekTo === 'function') {{ player.seekTo({}, true); }}", pos);
  }
  ```
- **Bug 1 (Scrub Flooding):** Scrubbing the seekbar emits dozens of IPC commands per second, queuing up in the wry webview event loop.
- **Bug 2 (Async State Lurch):** The webview takes 100-500ms to seek and report state back to Rust. In the meantime, `stream_controller.active_state.position_secs` retains the old timestamp, causing slider bounce-back.

---

## Summary Matrix of Defects

| Area | Defect | File Citation | Severity |
|---|---|---|---|
| **Area 2** | Seekbar hit rect bottom half (`[H - 7, H + 1]`) overlaps `RESIZE_MARGIN = 7.0px`. `handle_window_resize` sends `BeginResize(South)`, OS steals mouse events. | `src/ui/resize.rs:60-65`<br>`src/ui/playerbar.rs:540` | **Critical (Blocks UI input)** |
| **Area 3** | Rodio `ReadSeekSource` hardcodes `byte_len() -> None`. Symphonia FLAC demuxer requires stream length to seek. `sink.try_seek` returns `Err(SeekError::Unseekable)`. | `rodio-0.19.0\read_seek_source.rs:23`<br>`symphonia-bundle-flac\demuxer.rs:235` | **Critical (Blocks Audio seek)** |
| **Area 3** | `AudioEngine::seek` discards seek error with `let _ = sink.try_seek(pos);`. | `src/audio/engine.rs:764` | High |
| **Area 4** | `position()` is pure wall-clock time. Seek updates pause_offset while audio keeps playing, resulting in desync and "frozen bar at 100%". | `src/audio/engine.rs:776-785` | High |
| **Area 4** | Fake wall-clock timer triggers early gapless transition to track 2 while track 1 is still playing in hardware. | `src/audio/mod.rs:518-528` | High |
| **Area 4** | Seeking backwards does not clear pre-queued next track from sink or reset `self.prequeued`. | `src/audio/engine.rs:759-774` | Medium |
| **Area 4** | Seeking during crossfade takes `crossfade_sink` but leaves `self.sink` volume attenuated or zeroed. | `src/audio/engine.rs:760` | High |
| **Area 1** | Dragging seekbar calls seek on every frame (60Hz), spamming IPC and resetting timers. On drag release, snapback occurs. | `src/ui/playerbar.rs:555-570` | Medium |
| **Area 5** | Left/Right arrow keys ignore streaming playback and seek to 0s on streams. | `src/ui/shortcuts.rs:47-66` | Medium |
| **Area 5** | PiP mini seekbar does not support stream tracks (0.001s duration) and calls engine seek. | `src/ui/pip.rs:400-426` | Medium |

---

## Step 1 Implementation & Architectural Decision (Audio Engine & State)

### 1. Engine Decoding: Rodio vs. Native Symphonia Decoder Source
- **Evaluation of Rodio Upgrades:**
  - `rodio 0.19.0` and `rodio 0.20.1`: In both versions, `rodio::Decoder::new` wraps reader instances in an internal `ReadSeekSource` struct whose `byte_len(&self) -> Option<u64>` is hardcoded to return `None`. Symphonia's demuxers (FLAC, MP3 without Xing TOC, MP4/M4A) unconditionally require `reader.byte_len()` to compute seek table ranges and byte offsets; without it, `format.seek()` returns `SeekError(Unseekable)`. Rodio 0.19/0.20 exposes no API to pass a custom `MediaSourceStream` or specify `byte_len`.
  - `rodio 0.23.0`: Introduced `DecoderBuilder::with_byte_len()`, but broke backwards compatibility by completely removing `Sink`, `OutputStream`, and `OutputStreamHandle` in favor of an entirely different architecture (`DeviceSinkBuilder`, `MixerDeviceSink`, `Player`). Adopting rodio 0.23 would require rewriting CPAL output streams and sink handling across the entire app.
- **Architectural Decision:**
  - Implement `SymphoniaDecoderSource` in `src/audio/symphonia_decoder.rs`, implementing `rodio::Source<Item = f32> + Send + 'static`.
  - Because `std::fs::File` implements `symphonia::core::io::MediaSource` directly where `byte_len()` returns `Some(file.metadata().len())`, Symphonia demuxers (FLAC, WAV, MP3, AAC, M4A, OGG) can calculate seek tables and perform instantaneous, sample-accurate seeks.
  - Samples are decoded directly into `SampleBuffer<f32>`, upgrading lossless playback fidelity (24-bit and 32-bit audio files retain full float precision instead of 16-bit truncation).
  - A fallback to `rodio::Decoder::new` is retained for any exotic format not recognized by Symphonia.

### 2. Error Handling & Robust Fallback Seek
- `AudioEngine::seek` no longer discards seek errors. Errors are logged with `eprintln!` and stored in `self.last_seek_error` (accessible via `last_seek_error()`).
- If an in-place seek on the existing sink fails, `AudioEngine` executes fallback recreation: re-opens the file with a fresh decoder seeked to `pos` (and sample fast-forwarding if in-place seek is unindexed), replacing the sink seamlessly.
- State (`self.pause_offset`, `self.start_instant`) is ONLY updated after a confirmed successful seek (or fallback seek). If both fail, position state remains unchanged, preventing UI/audio desynchronization.

### 3. Position Clock & Frozen Bar Prevention
- `AudioEngine::position()` now prioritizes `sink.get_pos()`, which reads real-time hardware sample progress every 5ms from rodio's mixer thread.
- Clamping is bound to actual hardware playback, eliminating the bug where a fake wall-clock timer reached 100% and froze while audio continued playing from earlier offsets.

### 4. Crossfade & Gapless Prequeue State Fixes
- **Crossfade Interruption:** Seeking during an active crossfade immediately stops `crossfade_sink`, drops it to `None`, and restores `self.sink` volume to `self.volume` (or 0.0 if muted).
- **Gapless Prequeue Flush:** If `self.prequeued` is true when seeking, `AudioEngine::seek` resets `prequeued = false` and recreates playback for the current track seeked to `pos`, flushing the prequeued track so it will not play out of order.
- **Premature Transition Fix:** `has_track_reached_end()` now checks `sink.len() <= 1` when `prequeued` is active. Because rodio's `Done` wrapper decrements `sink.len()` only when track 1 yields `None` in the audio mixer thread, `transition_to_prequeued()` triggers at the exact microsecond track 2 begins rendering in hardware.
