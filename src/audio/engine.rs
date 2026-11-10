use crate::audio::symphonia_decoder::SymphoniaDecoderSource;
use crate::audio::visualizer::AudioTapBuffer;
use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Decodes an audio file into a sample source.
/// First attempts high-fidelity decoding and seeking via Symphonia. If that fails
/// (e.g. For unsupported exotic containers), falls back gracefully to rodio's decoder.
fn decode_audio_file(path: &Path) -> Result<Box<dyn Source<Item = f32> + Send>, String> {
    match SymphoniaDecoderSource::open_file(path) {
        Ok(source) => Ok(Box::new(source)),
        Err(symphonia_err) => {
            eprintln!(
                "Notice: Symphonia decoder could not open '{}' ({}); falling back to rodio decoder",
                path.display(),
                symphonia_err
            );
            let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;
            Ok(Box::new(source.convert_samples::<f32>()))
        }
    }
}

pub struct TapSource<S> {
    input: S,
    channels: u16,
    sample_rate: u32,
    chunk: Vec<f32>,
    tap_buffer: Arc<Mutex<AudioTapBuffer>>,
    normalization_bits: Arc<AtomicU32>,
}

impl<S: Source<Item = f32>> TapSource<S> {
    pub fn new(input: S, tap_buffer: Arc<Mutex<AudioTapBuffer>>) -> Self {
        Self::new_with_normalization(
            input,
            tap_buffer,
            Arc::new(AtomicU32::new(1.0f32.to_bits())),
        )
    }

    pub fn new_with_normalization(
        input: S,
        tap_buffer: Arc<Mutex<AudioTapBuffer>>,
        normalization_bits: Arc<AtomicU32>,
    ) -> Self {
        let channels = input.channels();
        let sample_rate = input.sample_rate();
        Self {
            input,
            channels,
            sample_rate,
            chunk: Vec::with_capacity(256),
            tap_buffer,
            normalization_bits,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for TapSource<S> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let sample = match self.input.next() {
            Some(s) => s,
            None => {
                if !self.chunk.is_empty() {
                    if let Ok(mut tap) = self.tap_buffer.try_lock() {
                        tap.push_interleaved(&self.chunk, self.channels, self.sample_rate);
                    }
                    self.chunk.clear();
                }
                return None;
            }
        };

        let norm_factor = f32::from_bits(self.normalization_bits.load(Ordering::Relaxed));
        let normalized = (sample * norm_factor).clamp(-1.0, 1.0);

        self.chunk.push(normalized);
        if self.chunk.len() >= 256 {
            if let Ok(mut tap) = self.tap_buffer.try_lock() {
                tap.push_interleaved(&self.chunk, self.channels, self.sample_rate);
            }
            self.chunk.clear();
        }

        Some(normalized)
    }
}

impl<S: Source<Item = f32>> Source for TapSource<S> {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        self.input.current_frame_len()
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.channels
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.input.total_duration()
    }

    #[inline]
    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        let res = self.input.try_seek(pos);
        if res.is_ok() {
            self.chunk.clear();
        }
        res
    }
}

/// Helper to calculate equal-power crossfade gains where t is in [0.0, 1.0].
/// Outgoing curve uses cos(t * pi / 2), incoming curve uses sin(t * pi / 2).
pub fn equal_power_crossfade(t: f32) -> (f32, f32) {
    let clamped = t.clamp(0.0, 1.0);
    let angle = clamped * std::f32::consts::FRAC_PI_2;
    (angle.cos(), angle.sin())
}

/// Helper for equal-power fade out curve: cos(t * pi / 2)
pub fn equal_power_fade_out(t: f32) -> f32 {
    let clamped = t.clamp(0.0, 1.0);
    (clamped * std::f32::consts::FRAC_PI_2).cos()
}

/// Helper for equal-power fade in curve: sin(t * pi / 2)
pub fn equal_power_fade_in(t: f32) -> f32 {
    let clamped = t.clamp(0.0, 1.0);
    (clamped * std::f32::consts::FRAC_PI_2).sin()
}

#[derive(Debug, Clone, Copy)]
pub enum FadeTransition {
    FadingOut { start: Instant, duration: Duration, base_vol: f32 },
    FadingIn { start: Instant, duration: Duration, target_vol: f32 },
}

/// Limits sample amplitude to avoid clipping above 0.0 dBFS (-1.0 to 1.0).
#[inline]
pub fn limit_sample(sample: f32) -> f32 {
    sample.clamp(-1.0, 1.0)
}

/// Parses a ReplayGain decibel string such as "-6.2 dB", "+1.5 dB", or "-4.0".
pub fn parse_replaygain_string(s: &str) -> Option<f32> {
    let clean = s.trim();
    let clean = clean
        .strip_suffix("dB")
        .or_else(|| clean.strip_suffix("db"))
        .or_else(|| clean.strip_suffix("DB"))
        .unwrap_or(clean)
        .trim();
    clean.parse::<f32>().ok()
}

/// Reads ReplayGain track and album gain tags from an audio file using lofty.
pub fn read_replaygain_from_file(path: &Path) -> (Option<f32>, Option<f32>) {
    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.read()) {
        if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
            let track_gain = tag
                .get(&ItemKey::ReplayGainTrackGain)
                .and_then(|item| item.value().text())
                .and_then(parse_replaygain_string);
            let album_gain = tag
                .get(&ItemKey::ReplayGainAlbumGain)
                .and_then(|item| item.value().text())
                .and_then(parse_replaygain_string);
            return (track_gain, album_gain);
        }
    }
    (None, None)
}

pub struct AudioEngine {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    crossfade_sink: Option<(Sink, Instant, Duration)>,
    volume: f32,
    muted: bool,
    start_instant: Option<Instant>,
    pause_offset: Duration,
    is_paused: bool,
    track_duration: Duration,
    current_path: Option<PathBuf>,
    current_duration_secs: f64,
    tap_buffer: Arc<Mutex<AudioTapBuffer>>,

    // Output device and buffer settings
    output_device_name: String,
    buffer_ms: u32,
    exclusive_mode: bool,

    // Click-free transition settings
    fade_pause_resume_ms: u32,
    fade_transition: Option<FadeTransition>,
    crossfade_sec: f32,
    gapless: bool,
    prequeued: bool,

    // Loudness and normalization settings
    volume_leveling_mode: String,
    smart_target_lufs: f32,
    current_track_lufs: Option<f32>,
    replaygain_track_db: Option<f32>,
    replaygain_album_db: Option<f32>,
    normalization_factor: f32,
    normalization_bits: Arc<AtomicU32>,

    // Seek status and error diagnostics
    last_seek_error: Option<String>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self::with_tap_buffer(AudioTapBuffer::new_shared())
    }

    pub fn with_tap_buffer(tap: Arc<Mutex<AudioTapBuffer>>) -> Self {
        let (stream, handle) = Self::create_output_stream("System default", 120, false);

        Self {
            _stream: stream,
            stream_handle: handle,
            sink: None,
            crossfade_sink: None,
            volume: 0.8,
            muted: false,
            start_instant: None,
            pause_offset: Duration::ZERO,
            is_paused: false,
            track_duration: Duration::ZERO,
            current_path: None,
            current_duration_secs: 0.0,
            tap_buffer: tap,
            output_device_name: "System default".to_string(),
            buffer_ms: 120,
            exclusive_mode: false,
            fade_pause_resume_ms: 180,
            fade_transition: None,
            crossfade_sec: 0.0,
            gapless: true,
            prequeued: false,
            volume_leveling_mode: "off".to_string(),
            smart_target_lufs: -14.0,
            current_track_lufs: None,
            replaygain_track_db: None,
            replaygain_album_db: None,
            normalization_factor: 1.0,
            normalization_bits: Arc::new(AtomicU32::new(1.0f32.to_bits())),
            last_seek_error: None,
        }
    }

    fn create_output_stream(
        device_name: &str,
        _buffer_ms: u32,
        _exclusive: bool,
    ) -> (Option<OutputStream>, Option<OutputStreamHandle>) {
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();

        if device_name != "System default" && !device_name.is_empty() {
            if let Ok(output_devices) = host.output_devices() {
                for dev in output_devices {
                    if let Ok(name) = dev.name() {
                        if name == device_name {
                            match OutputStream::try_from_device(&dev) {
                                Ok((s, h)) => return (Some(s), Some(h)),
                                Err(e) => {
                                    eprintln!(
                                        "Warning: failed to open stream for device '{}': {}. Falling back to default.",
                                        device_name, e
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Warning: could not open default audio output stream: {}", e);
                (None, None)
            }
        }
    }

    pub fn set_output_config(&mut self, device_name: &str, buffer_ms: u32, exclusive: bool) {
        let clamped_buffer = buffer_ms.clamp(20, 500);
        let device_changed = self.output_device_name != device_name;
        let buffer_changed = self.buffer_ms != clamped_buffer;
        let exclusive_changed = self.exclusive_mode != exclusive;

        if !device_changed && !buffer_changed && !exclusive_changed && self.stream_handle.is_some() {
            return;
        }

        self.output_device_name = device_name.to_string();
        self.buffer_ms = clamped_buffer;
        self.exclusive_mode = exclusive;

        let was_playing = self.is_playing();
        let was_paused = self.is_paused;
        let pos = self.position();
        let current_path = self.current_path.clone();
        let current_duration = self.current_duration_secs;

        let (new_stream, new_handle) =
            Self::create_output_stream(&self.output_device_name, self.buffer_ms, self.exclusive_mode);

        if let (Some(s), Some(h)) = (new_stream, new_handle) {
            if let Some(sink) = self.sink.take() {
                sink.stop();
            }
            if let Some((old_sink, _, _)) = self.crossfade_sink.take() {
                old_sink.stop();
            }

            self._stream = Some(s);
            self.stream_handle = Some(h);

            if let Some(path) = current_path {
                if was_playing || was_paused {
                    if let Err(e) = self.restore_playback(&path, current_duration, pos, was_paused) {
                        eprintln!("Warning: failed to restore playback after output config change: {}", e);
                    }
                }
            }
        }
    }

    fn restore_playback(
        &mut self,
        path: &Path,
        duration_secs: f64,
        pos: Duration,
        was_paused: bool,
    ) -> Result<(), String> {
        let mut source = decode_audio_file(path)?;

        if !pos.is_zero() {
            if let Err(e) = source.try_seek(pos) {
                eprintln!(
                    "Warning: source.try_seek to {:?} failed in restore_playback: {:?}. Fast-forwarding samples...",
                    pos, e
                );
                let channels = source.channels().max(1) as u64;
                let sample_rate = source.sample_rate().max(1) as u64;
                let samples_to_skip =
                    (pos.as_secs_f64() * sample_rate as f64 * channels as f64) as usize;
                for _ in 0..samples_to_skip {
                    if source.next().is_none() {
                        break;
                    }
                }
            }
        }

        if let Some(ref handle) = self.stream_handle {
            let new_sink = Sink::try_new(handle)
                .map_err(|e| format!("Failed to create audio sink: {}", e))?;

            let effective_vol = if self.muted { 0.0 } else { self.volume };
            new_sink.set_volume(effective_vol);

            let tapped = TapSource::new_with_normalization(
                source,
                self.tap_buffer.clone(),
                self.normalization_bits.clone(),
            );
            new_sink.append(tapped);

            if !pos.is_zero() {
                let _ = new_sink.try_seek(pos);
            }

            if was_paused {
                new_sink.pause();
                self.start_instant = None;
                self.pause_offset = pos;
                self.is_paused = true;
            } else {
                new_sink.play();
                self.start_instant = Some(Instant::now());
                self.pause_offset = pos;
                self.is_paused = false;
            }

            if let Some(old_sink) = self.sink.take() {
                old_sink.stop();
            }

            self.sink = Some(new_sink);
            self.track_duration = Duration::from_secs_f64(duration_secs);
            self.current_path = Some(path.to_path_buf());
            self.current_duration_secs = duration_secs;
            self.prequeued = false;
            self.fade_transition = None;
            self.last_seek_error = None;
            Ok(())
        } else {
            Err("No audio output device available".to_string())
        }
    }

    pub fn output_device_name(&self) -> &str {
        &self.output_device_name
    }

    pub fn buffer_ms(&self) -> u32 {
        self.buffer_ms
    }

    pub fn exclusive_mode(&self) -> bool {
        self.exclusive_mode
    }

    pub fn tap_buffer(&self) -> Arc<Mutex<AudioTapBuffer>> {
        self.tap_buffer.clone()
    }

    pub fn set_fade_pause_resume_ms(&mut self, ms: u32) {
        self.fade_pause_resume_ms = ms.clamp(0, 500);
    }

    pub fn fade_pause_resume_ms(&self) -> u32 {
        self.fade_pause_resume_ms
    }

    pub fn set_crossfade_sec(&mut self, sec: f32) {
        self.crossfade_sec = sec.clamp(0.0, 12.0);
    }

    pub fn crossfade_sec(&self) -> f32 {
        self.crossfade_sec
    }

    pub fn set_gapless(&mut self, enabled: bool) {
        self.gapless = enabled;
    }

    pub fn is_gapless(&self) -> bool {
        self.gapless
    }

    pub fn has_prequeued(&self) -> bool {
        self.prequeued
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.current_path.as_deref()
    }

    pub fn set_volume_leveling(
        &mut self,
        mode: &str,
        target_lufs: f32,
        current_track_lufs: Option<f32>,
        replaygain_track_db: Option<f32>,
        replaygain_album_db: Option<f32>,
    ) {
        self.volume_leveling_mode = mode.to_string();
        self.smart_target_lufs = target_lufs;
        self.current_track_lufs = current_track_lufs;
        self.replaygain_track_db = replaygain_track_db;
        self.replaygain_album_db = replaygain_album_db;

        let gain_db = match mode {
            "rg_track" => replaygain_track_db.unwrap_or(0.0),
            "rg_album" => replaygain_album_db.or(replaygain_track_db).unwrap_or(0.0),
            "smart_lufs" => {
                if let Some(track_lufs) = current_track_lufs {
                    target_lufs - track_lufs
                } else if let Some(rg_track) = replaygain_track_db {
                    (target_lufs + 18.0) + rg_track
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        let factor = if gain_db == 0.0 {
            1.0
        } else {
            10.0f32.powf(gain_db / 20.0).clamp(0.05, 8.0)
        };

        self.normalization_factor = factor;
        self.normalization_bits
            .store(factor.to_bits(), Ordering::Release);
    }

    pub fn normalization_factor(&self) -> f32 {
        self.normalization_factor
    }

    pub fn volume_leveling_mode(&self) -> &str {
        &self.volume_leveling_mode
    }

    pub fn smart_target_lufs(&self) -> f32 {
        self.smart_target_lufs
    }

    pub fn is_approaching_eof(&self, threshold: Duration) -> bool {
        if !self.is_playing() || self.track_duration.is_zero() {
            return false;
        }
        let pos = self.position();
        self.track_duration.saturating_sub(pos) <= threshold
    }

    pub fn prequeue_file(&mut self, path: &Path) -> Result<(), String> {
        if self.prequeued {
            return Ok(());
        }
        let source = decode_audio_file(path)?;

        if let Some(ref sink) = self.sink {
            let tapped = TapSource::new_with_normalization(
                source,
                self.tap_buffer.clone(),
                self.normalization_bits.clone(),
            );
            sink.append(tapped);
            self.prequeued = true;
            Ok(())
        } else {
            Err("No active sink to pre-queue".to_string())
        }
    }

    pub fn has_track_reached_end(&self) -> bool {
        if self.track_duration.is_zero() || self.is_paused {
            return false;
        }
        if self.prequeued {
            if let Some(ref sink) = self.sink {
                // If a track was prequeued, sink.len() was 2. When the first track
                // finishes playing in the audio thread, Done decrements sound_count to 1.
                return sink.len() <= 1;
            }
        }
        if let Some(ref sink) = self.sink {
            sink.empty()
        } else if let Some(start) = self.start_instant {
            start.elapsed() + self.pause_offset >= self.track_duration
        } else {
            false
        }
    }

    pub fn transition_to_prequeued(&mut self, next_path: &Path, duration_secs: f64) {
        self.current_path = Some(next_path.to_path_buf());
        self.current_duration_secs = duration_secs;
        self.track_duration = Duration::from_secs_f64(duration_secs);
        self.start_instant = Some(Instant::now());
        self.pause_offset = Duration::ZERO;
        self.prequeued = false;
        self.fade_transition = None;
        self.last_seek_error = None;
    }

    pub fn update_crossfade(&mut self) {
        if let Some((ref old_sink, start, duration)) = self.crossfade_sink {
            let elapsed = start.elapsed();
            let current_vol = if self.muted { 0.0 } else { self.volume };
            if elapsed < duration {
                let t = elapsed.as_secs_f32() / duration.as_secs_f32();
                let (out_gain, in_gain) = equal_power_crossfade(t);
                old_sink.set_volume(current_vol * out_gain);
                if let Some(ref new_sink) = self.sink {
                    new_sink.set_volume(current_vol * in_gain);
                }
            } else {
                old_sink.stop();
                self.crossfade_sink = None;
                if let Some(ref new_sink) = self.sink {
                    new_sink.set_volume(current_vol);
                }
            }
        }

        // Process non-blocking frame-based pause/resume fading
        if let Some(trans) = self.fade_transition {
            match trans {
                FadeTransition::FadingOut { start, duration, base_vol } => {
                    let elapsed = start.elapsed();
                    if elapsed < duration {
                        let t = elapsed.as_secs_f32() / duration.as_secs_f32();
                        let vol = (base_vol * (1.0 - t)).max(0.0);
                        if let Some(ref sink) = self.sink {
                            sink.set_volume(vol);
                        }
                    } else {
                        if let Some(ref sink) = self.sink {
                            sink.pause();
                            sink.set_volume(base_vol);
                        }
                        self.pause_offset = self.position();
                        self.start_instant = None;
                        self.is_paused = true;
                        self.fade_transition = None;
                    }
                }
                FadeTransition::FadingIn { start, duration, target_vol } => {
                    let elapsed = start.elapsed();
                    if elapsed < duration {
                        let t = elapsed.as_secs_f32() / duration.as_secs_f32();
                        let vol = (target_vol * t).min(target_vol);
                        if let Some(ref sink) = self.sink {
                            sink.set_volume(vol);
                        }
                    } else {
                        if let Some(ref sink) = self.sink {
                            sink.set_volume(target_vol);
                        }
                        self.fade_transition = None;
                    }
                }
            }
        }
    }

    pub fn play_file(&mut self, path: &Path, duration_secs: f64) -> Result<(), String> {
        let source = decode_audio_file(path)?;

        if let Some(ref handle) = self.stream_handle {
            let effective_vol = if self.muted { 0.0 } else { self.volume };

            if self.crossfade_sec > 0.0 && self.is_playing() {
                if let Some(old_sink) = self.sink.take() {
                    self.crossfade_sink = Some((
                        old_sink,
                        Instant::now(),
                        Duration::from_secs_f32(self.crossfade_sec),
                    ));
                }
            } else {
                if let Some(sink) = self.sink.take() {
                    sink.stop();
                }
                if let Some((old_sink, _, _)) = self.crossfade_sink.take() {
                    old_sink.stop();
                }
            }

            if let Ok(mut tap) = self.tap_buffer.lock() {
                tap.clear();
            }

            let new_sink = Sink::try_new(handle)
                .map_err(|e| format!("Failed to create audio sink: {}", e))?;

            if self.crossfade_sink.is_some() {
                new_sink.set_volume(0.0);
            } else {
                new_sink.set_volume(effective_vol);
            }

            let tapped = TapSource::new_with_normalization(
                source,
                self.tap_buffer.clone(),
                self.normalization_bits.clone(),
            );
            new_sink.append(tapped);
            new_sink.play();

            self.sink = Some(new_sink);
            self.current_path = Some(path.to_path_buf());
            self.current_duration_secs = duration_secs;
            self.track_duration = Duration::from_secs_f64(duration_secs);
            self.start_instant = Some(Instant::now());
            self.pause_offset = Duration::ZERO;
            self.is_paused = false;
            self.prequeued = false;
            self.last_seek_error = None;
            Ok(())
        } else {
            Err("No audio output device available".to_string())
        }
    }

    pub fn is_playing(&self) -> bool {
        if let Some(ref sink) = self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn is_finished(&self) -> bool {
        if let Some(ref sink) = self.sink {
            sink.empty() && self.crossfade_sink.is_none()
        } else {
            false
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            if !self.is_paused && !matches!(self.fade_transition, Some(FadeTransition::FadingOut { .. })) {
                let base_vol = if self.muted { 0.0 } else { self.volume };
                if self.fade_pause_resume_ms > 0 && !sink.empty() && base_vol > 0.0 {
                    self.fade_transition = Some(FadeTransition::FadingOut {
                        start: Instant::now(),
                        duration: Duration::from_millis(self.fade_pause_resume_ms as u64),
                        base_vol,
                    });
                } else {
                    sink.pause();
                    sink.set_volume(base_vol);
                    self.pause_offset = self.position();
                    self.start_instant = None;
                    self.is_paused = true;
                    self.fade_transition = None;
                }
            }
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref sink) = self.sink {
            if self.is_paused || matches!(self.fade_transition, Some(FadeTransition::FadingOut { .. })) {
                let base_vol = if self.muted { 0.0 } else { self.volume };
                if self.fade_pause_resume_ms > 0 && !sink.empty() && base_vol > 0.0 {
                    sink.set_volume(0.0);
                    sink.play();
                    self.start_instant = Some(Instant::now());
                    self.is_paused = false;
                    self.fade_transition = Some(FadeTransition::FadingIn {
                        start: Instant::now(),
                        duration: Duration::from_millis(self.fade_pause_resume_ms as u64),
                        target_vol: base_vol,
                    });
                } else {
                    sink.set_volume(base_vol);
                    sink.play();
                    self.start_instant = Some(Instant::now());
                    self.is_paused = false;
                    self.fade_transition = None;
                }
            }
        }
    }

    pub fn play(&mut self) {
        self.resume();
    }

    pub fn toggle_play_pause(&mut self) {
        if self.is_paused() || matches!(self.fade_transition, Some(FadeTransition::FadingOut { .. })) {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        if let Some((old_sink, _, _)) = self.crossfade_sink.take() {
            old_sink.stop();
        }
        if let Ok(mut tap) = self.tap_buffer.lock() {
            tap.clear();
        }
        self.start_instant = None;
        self.pause_offset = Duration::ZERO;
        self.is_paused = false;
        self.current_path = None;
        self.current_duration_secs = 0.0;
        self.prequeued = false;
        self.fade_transition = None;
        self.last_seek_error = None;
    }

    pub fn last_seek_error(&self) -> Option<&str> {
        self.last_seek_error.as_deref()
    }

    pub fn clear_seek_error(&mut self) {
        self.last_seek_error = None;
    }

    pub fn seek(&mut self, pos: Duration) {
        // 1. Crossfade interruption: stop outgoing crossfade sink and immediately
        // restore primary sink volume to target volume to avoid being stuck in an attenuated state.
        if let Some((old_sink, _, _)) = self.crossfade_sink.take() {
            old_sink.stop();
        }
        if let Some(ref sink) = self.sink {
            let effective_vol = if self.muted { 0.0 } else { self.volume };
            sink.set_volume(effective_vol);
        }
        self.fade_transition = None;

        // 2. Pre-queue flush: if a next track was already prequeued into the sink queue,
        // seeking in the current track requires discarding the prequeued track so it does not
        // start unexpectedly. Recreate playback for the current track seeked to `pos`.
        if self.prequeued {
            self.prequeued = false;
            if let Some(path) = self.current_path.clone() {
                let dur = self.current_duration_secs;
                let paused = self.is_paused;
                if let Err(e) = self.restore_playback(&path, dur, pos, paused) {
                    eprintln!("Warning: failed to restore playback on seek with prequeue: {}", e);
                    self.last_seek_error = Some(format!("Seek with prequeue failed: {}", e));
                } else {
                    self.last_seek_error = None;
                    if let Ok(mut tap) = self.tap_buffer.lock() {
                        tap.clear();
                    }
                    return;
                }
            }
        }

        // 3. In-place seek attempt on existing active sink (only if sink is not finished/empty)
        let mut seek_succeeded = false;
        if let Some(ref sink) = self.sink {
            if !sink.empty() {
                match sink.try_seek(pos) {
                    Ok(()) => {
                        seek_succeeded = true;
                        self.last_seek_error = None;
                    }
                    Err(err) => {
                        eprintln!(
                            "Warning: in-place sink.try_seek to {:?} failed: {:?}. Attempting fallback reopen-and-seek...",
                            pos, err
                        );
                    }
                }
            }
        }

        // 4. Robust fallback: if in-place seek failed or no sink was present, recreate the sink
        // and decode/fast-forward to the target position so seeking never silently fails.
        if !seek_succeeded {
            if let Some(path) = self.current_path.clone() {
                let dur = self.current_duration_secs;
                let paused = self.is_paused;
                match self.restore_playback(&path, dur, pos, paused) {
                    Ok(()) => {
                        seek_succeeded = true;
                        self.last_seek_error = None;
                    }
                    Err(err) => {
                        eprintln!("Error: fallback seek failed for '{}': {}", path.display(), err);
                        self.last_seek_error = Some(format!("Seek failed: {}", err));
                    }
                }
            }
        }

        // 5. Update position state ONLY after confirmed successful seek
        if seek_succeeded {
            self.pause_offset = pos;
            if !self.is_paused {
                self.start_instant = Some(Instant::now());
            } else {
                self.start_instant = None;
            }
            if let Ok(mut tap) = self.tap_buffer.lock() {
                tap.clear();
            }
        }
    }

    pub fn position(&self) -> Duration {
        if self.is_paused {
            self.pause_offset
        } else if let Some(ref sink) = self.sink {
            let sink_pos = sink.get_pos();
            if !sink_pos.is_zero() {
                sink_pos.min(self.track_duration)
            } else if let Some(start) = self.start_instant {
                (self.pause_offset + start.elapsed()).min(self.track_duration)
            } else {
                self.pause_offset
            }
        } else if let Some(start) = self.start_instant {
            (self.pause_offset + start.elapsed()).min(self.track_duration)
        } else {
            self.pause_offset
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if !self.muted {
            if let Some(ref sink) = self.sink {
                sink.set_volume(self.volume);
            }
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        let target = if self.muted { 0.0 } else { self.volume };
        if let Some(ref sink) = self.sink {
            sink.set_volume(target);
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn toggle_mute(&mut self) {
        self.set_muted(!self.muted);
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_engine_initialization() {
        let engine = AudioEngine::new();
        assert_eq!(engine.volume(), 0.8);
        assert!(!engine.is_muted());
        assert!(!engine.is_playing());
        assert!(!engine.is_paused());
        assert!(!engine.is_finished());
        assert_eq!(engine.output_device_name(), "System default");
        assert_eq!(engine.buffer_ms(), 120);
        assert!(!engine.exclusive_mode());
        assert_eq!(engine.fade_pause_resume_ms(), 180);
        assert_eq!(engine.crossfade_sec(), 0.0);
        assert!(engine.is_gapless());
        assert_eq!(engine.volume_leveling_mode(), "off");
        assert_eq!(engine.normalization_factor(), 1.0);
    }

    #[test]
    fn test_audio_engine_volume_and_mute() {
        let mut engine = AudioEngine::new();
        engine.set_volume(0.42);
        assert!((engine.volume() - 0.42).abs() < 1e-4);

        engine.toggle_mute();
        assert!(engine.is_muted());

        engine.toggle_mute();
        assert!(!engine.is_muted());
    }

    #[test]
    fn test_equal_power_crossfade_math() {
        let (out0, in0) = equal_power_crossfade(0.0);
        assert!((out0 - 1.0).abs() < 1e-5);
        assert!(in0.abs() < 1e-5);

        let (out1, in1) = equal_power_crossfade(1.0);
        assert!(out1.abs() < 1e-5);
        assert!((in1 - 1.0).abs() < 1e-5);

        for step in 0..=10 {
            let t = step as f32 / 10.0;
            let (out_g, in_g) = equal_power_crossfade(t);
            let sum_of_squares = out_g * out_g + in_g * in_g;
            assert!(
                (sum_of_squares - 1.0).abs() < 1e-4,
                "Equal-power property violated at t={}: out={}, in={}, sum={}",
                t,
                out_g,
                in_g,
                sum_of_squares
            );
        }

        assert_eq!(equal_power_fade_out(0.0), 1.0);
        assert!((equal_power_fade_in(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_limit_sample() {
        assert_eq!(limit_sample(0.5), 0.5);
        assert_eq!(limit_sample(-0.5), -0.5);
        assert_eq!(limit_sample(1.5), 1.0);
        assert_eq!(limit_sample(-2.0), -1.0);
        assert_eq!(limit_sample(1.0), 1.0);
        assert_eq!(limit_sample(-1.0), -1.0);
    }

    #[test]
    fn test_parse_replaygain_string() {
        assert_eq!(parse_replaygain_string("-6.2 dB"), Some(-6.2));
        assert_eq!(parse_replaygain_string("+1.50 dB"), Some(1.50));
        assert_eq!(parse_replaygain_string("-4.0db"), Some(-4.0));
        assert_eq!(parse_replaygain_string("2.5"), Some(2.5));
        assert_eq!(parse_replaygain_string("invalid"), None);
        assert_eq!(parse_replaygain_string(""), None);
    }

    #[test]
    fn test_volume_leveling_calculations() {
        let mut engine = AudioEngine::new();

        // Mode: off
        engine.set_volume_leveling("off", -14.0, Some(-18.0), Some(-4.0), Some(-4.0));
        assert_eq!(engine.normalization_factor(), 1.0);

        // Mode: rg_track with -6.0 dB gain -> factor = 10^(-6/20) ~ 0.501187
        engine.set_volume_leveling("rg_track", -14.0, None, Some(-6.0), None);
        assert!((engine.normalization_factor() - 0.501187).abs() < 1e-3);

        // Mode: rg_album with -3.0 dB track and -6.0 dB album -> factor ~ 0.501187
        engine.set_volume_leveling("rg_album", -14.0, None, Some(-3.0), Some(-6.0));
        assert!((engine.normalization_factor() - 0.501187).abs() < 1e-3);

        // Mode: smart_lufs with target -14.0 and current track -20.0 (+6 dB boost -> factor ~ 1.995)
        engine.set_volume_leveling("smart_lufs", -14.0, Some(-20.0), None, None);
        assert!((engine.normalization_factor() - 1.99526).abs() < 1e-3);

        // Mode: smart_lufs with target -14.0 and current track -10.0 (-4 dB cut -> factor ~ 0.6309)
        engine.set_volume_leveling("smart_lufs", -14.0, Some(-10.0), None, None);
        assert!((engine.normalization_factor() - 0.63095).abs() < 1e-3);
    }

    #[test]
    fn test_set_output_config_clamping_and_storage() {
        let mut engine = AudioEngine::new();
        engine.set_output_config("System default", 10, true);
        assert_eq!(engine.buffer_ms(), 20); // Clamped to min 20 ms
        assert!(engine.exclusive_mode());

        engine.set_output_config("System default", 999, false);
        assert_eq!(engine.buffer_ms(), 500); // Clamped to max 500 ms
        assert!(!engine.exclusive_mode());
    }

    #[test]
    fn test_transitions_configuration() {
        let mut engine = AudioEngine::new();
        engine.set_fade_pause_resume_ms(250);
        assert_eq!(engine.fade_pause_resume_ms(), 250);

        engine.set_fade_pause_resume_ms(999);
        assert_eq!(engine.fade_pause_resume_ms(), 500);

        engine.set_crossfade_sec(5.5);
        assert_eq!(engine.crossfade_sec(), 5.5);

        engine.set_crossfade_sec(15.0);
        assert_eq!(engine.crossfade_sec(), 12.0);

        engine.set_gapless(false);
        assert!(!engine.is_gapless());
        engine.set_gapless(true);
        assert!(engine.is_gapless());
    }

    #[test]
    fn test_position_math_and_clamp() {
        let mut engine = AudioEngine::new();
        engine.track_duration = Duration::from_secs(180);

        // Initially zero
        assert_eq!(engine.position(), Duration::ZERO);

        // Paused state
        engine.is_paused = true;
        engine.pause_offset = Duration::from_secs(45);
        assert_eq!(engine.position(), Duration::from_secs(45));

        // When paused, position does not advance with elapsed time
        assert_eq!(engine.position(), Duration::from_secs(45));

        // Clamping to track duration
        engine.pause_offset = Duration::from_secs(200);
        assert_eq!(engine.position(), Duration::from_secs(200)); // offset itself when paused
        engine.is_paused = false;
        engine.start_instant = Some(Instant::now());
        // When playing with offset >= duration, clamps to track_duration
        assert_eq!(engine.position(), Duration::from_secs(180));
    }

    #[test]
    fn test_crossfade_interruption_and_volume_restoration() {
        let mut engine = AudioEngine::new();
        engine.volume = 0.85;

        let (_stream, handle) = rodio::OutputStream::try_default().ok().unzip();
        if let Some(ref h) = handle {
            let primary_sink = Sink::try_new(h).unwrap();
            primary_sink.set_volume(0.2); // Simulated crossfade attenuation
            engine.sink = Some(primary_sink);

            let old_sink = Sink::try_new(h).unwrap();
            engine.crossfade_sink = Some((old_sink, Instant::now(), Duration::from_secs(5)));

            // Seek should immediately stop the old sink, clear crossfade_sink,
            // and restore primary sink volume to 0.85.
            engine.seek(Duration::from_secs(10));

            assert!(engine.crossfade_sink.is_none());
            if let Some(ref sink) = engine.sink {
                assert!((sink.volume() - 0.85).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn test_prequeue_flush_on_seek() {
        let mut engine = AudioEngine::new();
        engine.prequeued = true;
        engine.current_path = None; // No path to recreate

        // Calling seek while prequeued must reset prequeued to false
        engine.seek(Duration::from_secs(15));
        assert!(!engine.prequeued);
    }

    fn generate_test_wav_bytes(num_samples: usize, sample_rate: u32) -> Vec<u8> {
        let num_channels: u16 = 2;
        let bits_per_sample: u16 = 16;
        let block_align: u16 = num_channels * bits_per_sample / 8;
        let byte_rate: u32 = sample_rate * block_align as u32;
        let data_size: u32 = num_samples as u32 * block_align as u32;
        let file_size: u32 = 36 + data_size;

        let mut bytes = Vec::with_capacity(44 + data_size as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&file_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&num_channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());

        for i in 0..num_samples {
            let sample = ((i as f32 * 440.0 * 2.0 * std::f32::consts::PI / sample_rate as f32).sin()
                * 10000.0) as i16;
            bytes.extend_from_slice(&sample.to_le_bytes());
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        bytes
    }

    #[test]
    fn test_symphonia_and_engine_seek_generated_wav() {
        let temp_dir = std::env::temp_dir();
        let test_wav_path = temp_dir.join("eclipse_test_seek.wav");

        let wav_data = generate_test_wav_bytes(44100 * 10, 44100); // 10 seconds synthetic WAV
        std::fs::write(&test_wav_path, wav_data).expect("Failed to write test wav");

        // 1. Direct SymphoniaDecoderSource seek test
        let mut source = SymphoniaDecoderSource::open_file(&test_wav_path).expect("open_file wav");
        assert_eq!(source.channels(), 2);
        assert_eq!(source.sample_rate(), 44100);

        let res = source.try_seek(Duration::from_millis(1500));
        assert!(res.is_ok(), "Seeking generated WAV failed: {:?}", res.err());

        // 2. AudioEngine play and seek comprehensive test
        let mut engine = AudioEngine::new();
        engine.set_fade_pause_resume_ms(0);
        let play_res = engine.play_file(&test_wav_path, 10.0);
        if play_res.is_ok() {
            assert!(engine.is_playing());

            // Forward seek
            engine.seek(Duration::from_secs(4));
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(60));
            assert!(engine.position() >= Duration::from_secs(4));

            // Backward seek
            engine.seek(Duration::from_secs(1));
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(60));
            assert!(engine.position() >= Duration::from_secs(1) && engine.position() < Duration::from_secs(3));

            // Seek to 0s
            engine.seek(Duration::ZERO);
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(60));
            assert!(engine.position() < Duration::from_secs(2));

            // Seek near end
            engine.seek(Duration::from_secs(9));
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(60));
            assert!(engine.position() >= Duration::from_secs(9));

            // Paused seek
            engine.pause();
            assert!(engine.is_paused());
            engine.seek(Duration::from_secs(5));
            assert!(engine.is_paused());
            assert_eq!(engine.position(), Duration::from_secs(5));
            engine.resume();
            std::thread::sleep(Duration::from_millis(60));
            assert!(engine.position() >= Duration::from_secs(5));

            // Rapid repeated seeks
            for &pt in &[Duration::from_secs(2), Duration::from_secs(7), Duration::from_secs(3), Duration::from_secs(8)] {
                engine.seek(pt);
                assert!(engine.last_seek_error().is_none());
            }

            // Seek on finished track
            if let Some(ref sink) = engine.sink {
                sink.stop();
            }
            std::thread::sleep(Duration::from_millis(50));
            assert!(engine.is_finished());
            engine.seek(Duration::from_secs(2));
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(60));
            assert!(engine.position() >= Duration::from_secs(2));
            assert!(!engine.is_finished());

            engine.stop();
        }

        let _ = std::fs::remove_file(&test_wav_path);
    }

    #[test]
    fn test_real_flac_seeking_in_audio_engine() {
        let path = Path::new(r"C:\music\Muse\Aimyon - Marigold.flac");
        if !path.exists() {
            return;
        }

        let mut engine = AudioEngine::new();
        engine.set_fade_pause_resume_ms(0); // Immediate pause/resume for testing
        let play_res = engine.play_file(path, 306.0);
        if play_res.is_ok() {
            assert!(engine.is_playing());

            // Seek to 60s
            engine.seek(Duration::from_secs(60));
            assert!(engine.last_seek_error().is_none());
            assert_eq!(engine.pause_offset, Duration::from_secs(60));

            // Small delay to let sink advance
            std::thread::sleep(Duration::from_millis(50));
            let pos = engine.position();
            assert!(
                pos >= Duration::from_secs(60),
                "Position after seek should be >= 60s, got {:?}",
                pos
            );

            // Seek while paused
            engine.pause();
            assert!(engine.is_paused());
            engine.seek(Duration::from_secs(120));
            assert_eq!(engine.pause_offset, Duration::from_secs(120));
            assert_eq!(engine.position(), Duration::from_secs(120));

            engine.stop();
        }
    }

    #[test]
    fn test_empirical_audio_seeking_comprehensive() {
        let flac_paths = [
            r"C:\music\Muse\Aimyon - Marigold.flac",
            r"C:\music\Muse\Coldplay - Yellow.flac",
            r"C:\music\Muse\Eve - Dramaturgy.flac",
        ];

        for path_str in flac_paths {
            let path = Path::new(path_str);
            if !path.exists() {
                continue;
            }

            let source_probe = SymphoniaDecoderSource::open_file(path).expect("open_file failed");
            let total_dur = source_probe.total_duration().unwrap_or(Duration::from_secs(200));
            let total_dur_secs = total_dur.as_secs_f64();

            println!("\n=== Empirical Seek Verification for {} (dur: {:.2}s) ===", path.display(), total_dur_secs);
            let mut engine = AudioEngine::new();
            engine.set_fade_pause_resume_ms(0); // instant transitions for tests
            let play_res = engine.play_file(path, total_dur_secs);
            if play_res.is_err() {
                println!("Skipping path {} due to audio device error: {:?}", path_str, play_res.err());
                continue;
            }

            assert!(engine.is_playing());

            // 1. Initial play check
            std::thread::sleep(Duration::from_millis(60));
            let init_pos = engine.position();
            println!("Initial position: {:?}", init_pos);

            // 2. Forward seek (to 45.0s)
            println!("Seeking forward to 45.0s...");
            engine.seek(Duration::from_secs(45));
            assert!(engine.last_seek_error().is_none(), "Seek forward produced error: {:?}", engine.last_seek_error());
            std::thread::sleep(Duration::from_millis(80));
            let fwd_pos = engine.position();
            println!("Position after forward seek: {:?}", fwd_pos);
            assert!(fwd_pos >= Duration::from_secs(45), "Expected >= 45s, got {:?}", fwd_pos);
            assert!(fwd_pos < Duration::from_secs(50), "Expected < 50s, got {:?}", fwd_pos);

            // 3. Backward seek (from ~45s back to 10.0s)
            println!("Seeking backward to 10.0s...");
            engine.seek(Duration::from_secs(10));
            assert!(engine.last_seek_error().is_none(), "Seek backward produced error: {:?}", engine.last_seek_error());
            std::thread::sleep(Duration::from_millis(80));
            let back_pos = engine.position();
            println!("Position after backward seek: {:?}", back_pos);
            assert!(back_pos >= Duration::from_secs(10), "Expected >= 10s, got {:?}", back_pos);
            assert!(back_pos < Duration::from_secs(15), "Expected < 15s, got {:?}", back_pos);

            // 4. Seek to start (0.0s)
            println!("Seeking to start (0.0s)...");
            engine.seek(Duration::ZERO);
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(80));
            let start_pos = engine.position();
            println!("Position after seek to 0.0s: {:?}", start_pos);
            assert!(start_pos < Duration::from_secs(5), "Expected < 5s, got {:?}", start_pos);

            // 5. Seek near end (total_dur - 3.0s)
            let near_end = Duration::from_secs_f64((total_dur_secs - 3.0).max(1.0));
            println!("Seeking near end ({:?})...", near_end);
            engine.seek(near_end);
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(80));
            let end_pos = engine.position();
            println!("Position after seek near end: {:?}", end_pos);
            assert!(end_pos >= near_end, "Expected >= {:?}, got {:?}", near_end, end_pos);

            // 6. Seek while paused & verify paused state is retained
            println!("Pausing and seeking while paused...");
            engine.pause();
            assert!(engine.is_paused(), "Engine should be paused");
            std::thread::sleep(Duration::from_millis(30));

            engine.seek(Duration::from_secs(65));
            assert!(engine.is_paused(), "Engine should remain paused after seek");
            assert_eq!(engine.position(), Duration::from_secs(65), "Position should update to 65s while paused");
            std::thread::sleep(Duration::from_millis(60));
            // Ensure paused position does not advance on its own
            assert_eq!(engine.position(), Duration::from_secs(65), "Paused position must not drift");

            // Resume playback
            println!("Resuming playback from 65.0s...");
            engine.resume();
            assert!(!engine.is_paused());
            std::thread::sleep(Duration::from_millis(80));
            let resumed_pos = engine.position();
            println!("Position after resume: {:?}", resumed_pos);
            assert!(resumed_pos >= Duration::from_secs(65), "Position after resume should be >= 65s, got {:?}", resumed_pos);

            // 7. Rapid repeated seeks (stress test / scrub simulation)
            println!("Stress testing rapid repeated seeks...");
            let scrub_points = [
                Duration::from_secs(5),
                Duration::from_secs(12),
                Duration::from_secs(30),
                Duration::from_secs(18),
                Duration::from_secs(85),
                Duration::from_secs(42),
                Duration::from_secs(110),
                Duration::from_secs(2),
                Duration::from_secs(77),
                Duration::from_secs(150),
            ];
            for &pt in &scrub_points {
                engine.seek(pt);
                assert!(engine.last_seek_error().is_none());
            }
            // Final seek to 50s
            engine.seek(Duration::from_secs(50));
            std::thread::sleep(Duration::from_millis(80));
            let final_pos = engine.position();
            println!("Position after rapid scrubbing: {:?}", final_pos);
            assert!(final_pos >= Duration::from_secs(50) && final_pos < Duration::from_secs(55));

            // 8. Crossfade volume recovery verification
            println!("Testing crossfade volume recovery...");
            engine.set_volume(0.8);
            if let Some(ref sink) = engine.sink {
                sink.set_volume(0.1); // simulated crossfade dip
            }
            let (_dummy_stream, dummy_handle) = rodio::OutputStream::try_default().ok().unzip();
            if let Some(ref h) = dummy_handle {
                if let Ok(old_sink) = Sink::try_new(h) {
                    engine.crossfade_sink = Some((old_sink, Instant::now(), Duration::from_secs(5)));
                }
            }
            engine.seek(Duration::from_secs(20));
            assert!(engine.crossfade_sink.is_none(), "crossfade_sink should be None after seek");
            if let Some(ref sink) = engine.sink {
                assert!((sink.volume() - 0.8).abs() < 1e-4, "Primary sink volume must be restored to 0.8");
            }

            // 9. Prequeue flush verification
            println!("Testing prequeue flush on seek...");
            engine.prequeued = true;
            engine.seek(Duration::from_secs(25));
            assert!(!engine.prequeued, "prequeued must be reset to false after seek");
            std::thread::sleep(Duration::from_millis(80));
            let prequeue_pos = engine.position();
            println!("Position after prequeue flush seek: {:?}", prequeue_pos);
            assert!(prequeue_pos >= Duration::from_secs(25) && prequeue_pos < Duration::from_secs(30));

            // 10. Empty sink seek recovery (seeking after track finished/EOF)
            println!("Testing seek on empty/finished track...");
            if let Some(ref sink) = engine.sink {
                sink.stop(); // simulate EOF / finished track
            }
            std::thread::sleep(Duration::from_millis(50));
            assert!(engine.is_finished());
            engine.seek(Duration::from_secs(15));
            assert!(engine.last_seek_error().is_none());
            std::thread::sleep(Duration::from_millis(80));
            let restarted_pos = engine.position();
            println!("Position after seek on finished track: {:?}", restarted_pos);
            assert!(restarted_pos >= Duration::from_secs(15) && restarted_pos < Duration::from_secs(20));
            assert!(!engine.is_finished());

            engine.stop();
            println!("=== Verification for {} PASSED ===", path.display());
        }
    }
}
