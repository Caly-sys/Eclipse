# Task 3A Handoff: Real-Time Audio DSP, 2048 Hann FFT, Live Metering & Queue Progression

## Overview
Replaced all simulated/synthetic waveform and frequency generation with a real-time PCM audio streaming pipeline tapped directly from the Rodio decoder stream. Implemented 2048-point Hann-windowed forward FFT using `rustfft`, mapped magnitudes into logarithmic frequency bins (20Hz - 20kHz) with pink-noise tilt and temporal gravity decay, streamed true time-domain PCM samples to the Scope visualizer, and computed live L/R stereo RMS and Peak dBFS for the hardware meters. Fixed Next/Previous queue progression, shuffle indexing, repeat loop modes, and automatic EOF track transitions.

## Key Changes
1. **Real-Time PCM Sample Tap** (`src/audio/engine.rs` & `src/audio/visualizer.rs`):
   - Created `TapSource<S>: rodio::Source<Item = f32>` which wraps the audio decoder source and buffers interleaved PCM chunks without blocking the audio thread using non-blocking try-locks.
   - Implemented `AudioTapBuffer` maintaining stereo L/R and mono-downmixed sample queues up to 8192 samples.
   - Connected `AudioEngine::play_file` to wrap decoded sources in `TapSource` and stream live samples to `tap_buffer`.

2. **Studio-Grade 2048 Hann FFT & Logarithmic Spectrum** (`src/audio/visualizer.rs`):
   - Planned 2048-sample forward FFT via `rustfft::FftPlanner` cached in `Visualizer`.
   - Pre-computed Hann window coefficients of size 2048 to eliminate runtime trigonometry calculations.
   - Mapped FFT output into logarithmic frequency bins spanning 20Hz to 20kHz with high-frequency pink-noise compensation (+3dB/octave above 1kHz) and converted to dBFS (-60 dBFS to 0 dBFS range).
   - Applied asymmetric attack (26.0 * dt) and organic gravity decay (8.5 * dt) with peak-hold decay (0.45 * dt).
   - Streamed true PCM time-domain samples to `wave_buffer` for the Scope visualizer.
   - Computed real-time RMS, True Peak dBFS, estimated LUFS, and Crest Factor (Dynamic Range) from live PCM frames.
   - Smoothly decays all spectrum bars, scope waveforms, and meter levels to silence on pause/stop.
   - STRICT COMPLIANCE: Zero `Math.sin` or random noise generators for audio visualizers.

3. **Queue Progression, Next / Previous, Shuffle & Repeat** (`src/audio/mod.rs` & `src/ui/mod.rs`):
   - `load_queue` populates active queue (`Vec<Track>`), sets `current_index`, and prepares shuffle indices if shuffle is active.
   - `next()` advances through the active queue sequentially or follows `shuffled_indices` without replaying the current track; respects `RepeatMode::All` by looping/reshuffling or stops cleanly when `RepeatMode::Off`.
   - `prev()` retreats through queue/shuffle or seeks to 0:00 if track position is >= 3 seconds.
   - `update()` checks for EOF via `sink.empty() && !is_paused()` and automatically advances to the next track or repeats the current track when `RepeatMode::One`.
   - Fixed `on_play_track` dispatch in `src/ui/mod.rs` and `src/ui/views/home.rs` to load context queues rather than single-track vectors.
   - STRICT COMPLIANCE: Zero emojis added to any touched codebase files.

4. **Synchronized Repositories & Test Verification**:
   - Both `C:\music\umbra` and `C:\Users\Hype AMD\umbra` pass all 34 unit tests.
   - Unit tests covering `test_audio_tap_buffer_metrics`, `test_visualizer_initialization`, `test_visualizer_update_playback`, `test_load_queue_and_progression`, `test_repeat_all_loops_queue`, `test_shuffle_toggle_and_progression`, `test_cycle_repeat_modes`, `test_audio_engine_initialization`, and `test_audio_engine_volume_and_mute`.
