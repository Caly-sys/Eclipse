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
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;
        let f32_source = source.convert_samples::<f32>();

        if let Some(ref handle) = self.stream_handle {
            let new_sink = Sink::try_new(handle)
                .map_err(|e| format!("Failed to create audio sink: {}", e))?;

            let effective_vol = if self.muted { 0.0 } else { self.volume };
            new_sink.set_volume(effective_vol);

            let tapped = TapSource::new_with_normalization(
                f32_source,
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

            self.sink = Some(new_sink);
            self.track_duration = Duration::from_secs_f64(duration_secs);
            self.current_path = Some(path.to_path_buf());
            self.current_duration_secs = duration_secs;
            self.prequeued = false;
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
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;
        let f32_source = source.convert_samples::<f32>();

        if let Some(ref sink) = self.sink {
            let tapped = TapSource::new_with_normalization(
                f32_source,
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
        if let Some(start) = self.start_instant {
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
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;
        let f32_source = source.convert_samples::<f32>();

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
                f32_source,
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
    }

    pub fn seek(&mut self, pos: Duration) {
        if let Some((old_sink, _, _)) = self.crossfade_sink.take() {
            old_sink.stop();
        }
        if let Some(ref sink) = self.sink {
            let _ = sink.try_seek(pos);
            self.pause_offset = pos;
            if !self.is_paused {
                self.start_instant = Some(Instant::now());
            }
            if let Ok(mut tap) = self.tap_buffer.lock() {
                tap.clear();
            }
            self.fade_transition = None;
        }
    }

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
}
