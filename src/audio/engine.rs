use crate::audio::visualizer::AudioTapBuffer;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct TapSource<S> {
    input: S,
    channels: u16,
    sample_rate: u32,
    chunk: Vec<f32>,
    tap_buffer: Arc<Mutex<AudioTapBuffer>>,
}

impl<S: Source<Item = f32>> TapSource<S> {
    pub fn new(input: S, tap_buffer: Arc<Mutex<AudioTapBuffer>>) -> Self {
        let channels = input.channels();
        let sample_rate = input.sample_rate();
        Self {
            input,
            channels,
            sample_rate,
            chunk: Vec::with_capacity(256),
            tap_buffer,
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

        self.chunk.push(sample);
        if self.chunk.len() >= 256 {
            if let Ok(mut tap) = self.tap_buffer.try_lock() {
                tap.push_interleaved(&self.chunk, self.channels, self.sample_rate);
            }
            self.chunk.clear();
        }

        Some(sample)
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

pub struct AudioEngine {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    sink: Option<Sink>,
    volume: f32,
    muted: bool,
    start_instant: Option<Instant>,
    pause_offset: Duration,
    is_paused: bool,
    track_duration: Duration,
    tap_buffer: Arc<Mutex<AudioTapBuffer>>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self::with_tap_buffer(AudioTapBuffer::new_shared())
    }

    pub fn with_tap_buffer(tap: Arc<Mutex<AudioTapBuffer>>) -> Self {
        let (stream, handle) = match OutputStream::try_default() {
            Ok((s, h)) => (Some(s), Some(h)),
            Err(e) => {
                eprintln!("Warning: could not open default audio output stream: {}", e);
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle: handle,
            sink: None,
            volume: 0.8,
            muted: false,
            start_instant: None,
            pause_offset: Duration::ZERO,
            is_paused: false,
            track_duration: Duration::ZERO,
            tap_buffer: tap,
        }
    }

    pub fn tap_buffer(&self) -> Arc<Mutex<AudioTapBuffer>> {
        self.tap_buffer.clone()
    }

    pub fn play_file(&mut self, path: &Path, duration_secs: f64) -> Result<(), String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Failed to decode audio: {}", e))?;
        let f32_source = source.convert_samples::<f32>();

        if let Some(ref handle) = self.stream_handle {
            // Stop and drop previous sink
            if let Some(sink) = self.sink.take() {
                sink.stop();
            }

            if let Ok(mut tap) = self.tap_buffer.lock() {
                tap.clear();
            }

            let new_sink = Sink::try_new(handle)
                .map_err(|e| format!("Failed to create audio sink: {}", e))?;

            let effective_vol = if self.muted { 0.0 } else { self.volume };
            new_sink.set_volume(effective_vol);

            let tapped = TapSource::new(f32_source, self.tap_buffer.clone());
            new_sink.append(tapped);
            new_sink.play();

            self.sink = Some(new_sink);
            self.track_duration = Duration::from_secs_f64(duration_secs);
            self.start_instant = Some(Instant::now());
            self.pause_offset = Duration::ZERO;
            self.is_paused = false;
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
            sink.empty()
        } else {
            false
        }
    }

    pub fn pause(&mut self) {
        if let Some(ref sink) = self.sink {
            if !self.is_paused {
                sink.pause();
                self.pause_offset = self.position();
                self.start_instant = None;
                self.is_paused = true;
            }
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref sink) = self.sink {
            if self.is_paused {
                sink.play();
                self.start_instant = Some(Instant::now());
                self.is_paused = false;
            }
        }
    }

    pub fn toggle_play_pause(&mut self) {
        if self.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        if let Ok(mut tap) = self.tap_buffer.lock() {
            tap.clear();
        }
        self.start_instant = None;
        self.pause_offset = Duration::ZERO;
        self.is_paused = false;
    }

    pub fn seek(&mut self, pos: Duration) {
        if let Some(ref sink) = self.sink {
            let _ = sink.try_seek(pos);
            self.pause_offset = pos;
            if !self.is_paused {
                self.start_instant = Some(Instant::now());
            }
            if let Ok(mut tap) = self.tap_buffer.lock() {
                tap.clear();
            }
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
        if let Some(ref sink) = self.sink {
            if !self.muted {
                sink.set_volume(self.volume);
            }
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if let Some(ref sink) = self.sink {
            if self.muted {
                sink.set_volume(0.0);
            } else {
                sink.set_volume(self.volume);
            }
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
}
