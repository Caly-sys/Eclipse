use crate::config::VisualizerStyle;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const FFT_SIZE: usize = 2048;
pub const MAX_TAP_BUFFER_SAMPLES: usize = 8192;
pub const WAVE_BUFFER_SIZE: usize = 128;

#[derive(Clone, Debug)]
pub struct AudioTapBuffer {
    pub channels: u16,
    pub sample_rate: u32,
    pub samples_mono: VecDeque<f32>,
    pub samples_left: VecDeque<f32>,
    pub samples_right: VecDeque<f32>,
}

impl AudioTapBuffer {
    pub fn new() -> Self {
        Self {
            channels: 2,
            sample_rate: 44100,
            samples_mono: VecDeque::with_capacity(MAX_TAP_BUFFER_SAMPLES),
            samples_left: VecDeque::with_capacity(MAX_TAP_BUFFER_SAMPLES),
            samples_right: VecDeque::with_capacity(MAX_TAP_BUFFER_SAMPLES),
        }
    }

    pub fn new_shared() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }

    pub fn push_interleaved(&mut self, chunk: &[f32], channels: u16, sample_rate: u32) {
        if chunk.is_empty() {
            return;
        }
        self.channels = channels.max(1);
        self.sample_rate = if sample_rate > 0 { sample_rate } else { 44100 };
        let ch = self.channels as usize;

        let frames = chunk.len() / ch;
        for f in 0..frames {
            let offset = f * ch;
            let (left, right, mono) = match ch {
                1 => {
                    let s = chunk[offset];
                    (s, s, s)
                }
                2 => {
                    let l = chunk[offset];
                    let r = chunk[offset + 1];
                    let m = (l + r) * 0.5;
                    (l, r, m)
                }
                _ => {
                    let l = chunk[offset];
                    let r = chunk[offset + 1];
                    let mut sum = 0.0;
                    for c in 0..ch {
                        sum += chunk[offset + c];
                    }
                    (l, r, sum / ch as f32)
                }
            };

            self.samples_left.push_back(left);
            self.samples_right.push_back(right);
            self.samples_mono.push_back(mono);
        }

        while self.samples_mono.len() > MAX_TAP_BUFFER_SAMPLES {
            self.samples_mono.pop_front();
        }
        while self.samples_left.len() > MAX_TAP_BUFFER_SAMPLES {
            self.samples_left.pop_front();
        }
        while self.samples_right.len() > MAX_TAP_BUFFER_SAMPLES {
            self.samples_right.pop_front();
        }
    }

    pub fn clear(&mut self) {
        self.samples_mono.clear();
        self.samples_left.clear();
        self.samples_right.clear();
    }

    pub fn get_recent_mono(&self, count: usize) -> Vec<f32> {
        let total = self.samples_mono.len();
        let mut out = Vec::with_capacity(count);
        if total < count {
            let pad = count - total;
            out.resize(pad, 0.0);
            for &s in self.samples_mono.iter() {
                out.push(s);
            }
        } else {
            let skip = total - count;
            for &s in self.samples_mono.iter().skip(skip) {
                out.push(s);
            }
        }
        out
    }

    pub fn compute_stereo_metrics(&self, window_size: usize) -> (f32, f32, f32, f32) {
        let count_l = self.samples_left.len();
        let count_r = self.samples_right.len();
        if count_l == 0 || count_r == 0 {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let n = window_size.min(count_l).min(count_r);
        if n == 0 {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let skip_l = count_l - n;
        let skip_r = count_r - n;

        let mut sum_sq_l = 0.0f32;
        let mut peak_l = 0.0f32;
        for &s in self.samples_left.iter().skip(skip_l) {
            let abs_s = s.abs();
            if abs_s > peak_l {
                peak_l = abs_s;
            }
            sum_sq_l += s * s;
        }
        let rms_l = (sum_sq_l / n as f32).sqrt();

        let mut sum_sq_r = 0.0f32;
        let mut peak_r = 0.0f32;
        for &s in self.samples_right.iter().skip(skip_r) {
            let abs_s = s.abs();
            if abs_s > peak_r {
                peak_r = abs_s;
            }
            sum_sq_r += s * s;
        }
        let rms_r = (sum_sq_r / n as f32).sqrt();

        (rms_l, rms_r, peak_l, peak_r)
    }
}

impl Default for AudioTapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Visualizer {
    pub style: VisualizerStyle,
    pub bands_count: usize,
    pub band_values: Vec<f32>,
    pub peak_values: Vec<f32>,
    pub wave_buffer: Vec<f32>,
    pub left_level: f32,
    pub right_level: f32,
    pub left_peak: f32,
    pub right_peak: f32,
    pub peak_dbfs: f32,
    pub lufs: f32,
    pub dynamic_range: f32,
    tap_buffer: Arc<Mutex<AudioTapBuffer>>,
    fft: Arc<dyn Fft<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
    last_update: Instant,
}

impl Visualizer {
    pub fn new(style: VisualizerStyle, bands_count: usize, tap_buffer: Arc<Mutex<AudioTapBuffer>>) -> Self {
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);

        let mut window = Vec::with_capacity(FFT_SIZE);
        for i in 0..FFT_SIZE {
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32).cos());
            window.push(w);
        }

        Self {
            style,
            bands_count,
            band_values: vec![0.0; bands_count],
            peak_values: vec![0.0; bands_count],
            wave_buffer: vec![0.0; WAVE_BUFFER_SIZE],
            left_level: 0.0,
            right_level: 0.0,
            left_peak: 0.0,
            right_peak: 0.0,
            peak_dbfs: -60.0,
            lufs: -60.0,
            dynamic_range: 0.0,
            tap_buffer,
            fft,
            fft_buffer: vec![Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            window,
            last_update: Instant::now(),
        }
    }

    pub fn tap_buffer(&self) -> Arc<Mutex<AudioTapBuffer>> {
        self.tap_buffer.clone()
    }

    pub fn update(&mut self, is_playing: bool, _position_secs: f64) {
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32().clamp(0.001, 0.1);
        self.last_update = now;

        if !is_playing {
            // Decay to zero smoothly
            for v in &mut self.band_values {
                *v = (*v - dt * 2.5).max(0.0);
            }
            for p in &mut self.peak_values {
                *p = (*p - dt * 1.0).max(0.0);
            }
            for w in &mut self.wave_buffer {
                *w = if *w > 0.0 {
                    (*w - dt * 3.0).max(0.0)
                } else {
                    (*w + dt * 3.0).min(0.0)
                };
            }
            self.left_level = (self.left_level - dt * 3.0).max(0.0);
            self.right_level = (self.right_level - dt * 3.0).max(0.0);
            self.left_peak = (self.left_peak - dt * 1.0).max(self.left_level);
            self.right_peak = (self.right_peak - dt * 1.0).max(self.right_level);
            self.peak_dbfs = -60.0;
            self.lufs = -60.0;
            self.dynamic_range = 0.0;
            return;
        }

        let (recent_mono, stereo_metrics, sample_rate) = {
            if let Ok(tap) = self.tap_buffer.try_lock() {
                (
                    tap.get_recent_mono(FFT_SIZE),
                    tap.compute_stereo_metrics(1024),
                    tap.sample_rate,
                )
            } else {
                return;
            }
        };

        // Window and copy into pre-allocated FFT buffer
        for (i, (&s, &w)) in recent_mono.iter().zip(self.window.iter()).enumerate() {
            self.fft_buffer[i] = Complex {
                re: s * w,
                im: 0.0,
            };
        }

        // Real forward FFT
        self.fft.process(&mut self.fft_buffer);

        // Map FFT magnitudes to logarithmic frequency bins from 20Hz to 20kHz
        let count = self.bands_count;
        let sr = if sample_rate > 0 { sample_rate as f32 } else { 44100.0 };
        let half_n = FFT_SIZE / 2;
        let min_freq = 20.0f32;
        let max_freq = (sr * 0.5).min(20000.0);
        let freq_ratio = max_freq / min_freq;

        for i in 0..count {
            let f_start = min_freq * freq_ratio.powf(i as f32 / count as f32);
            let f_end = min_freq * freq_ratio.powf((i + 1) as f32 / count as f32);

            let bin_start = ((f_start * FFT_SIZE as f32 / sr).floor() as usize).clamp(1, half_n - 1);
            let bin_end = ((f_end * FFT_SIZE as f32 / sr).ceil() as usize).clamp(bin_start + 1, half_n);

            let mut max_mag = 0.0f32;
            for k in bin_start..bin_end {
                let c = self.fft_buffer[k];
                let mag = (c.re * c.re + c.im * c.im).sqrt();
                if mag > max_mag {
                    max_mag = mag;
                }
            }

            // Normalization & gentle high-frequency tilt (+3dB/octave above 1kHz)
            let norm_mag = (max_mag * 2.0) / FFT_SIZE as f32;
            let center_freq = (f_start * f_end).sqrt();
            let tilt = (center_freq / 1000.0).powf(0.25).clamp(0.8, 3.5);
            let weighted_mag = norm_mag * tilt;

            // Map dBFS (-60 dBFS to 0 dBFS) to [0.0, 1.0]
            let db = if weighted_mag > 1e-5 {
                20.0 * weighted_mag.log10()
            } else {
                -60.0
            };
            let target = ((db + 60.0) / 60.0).clamp(0.0, 1.0);

            // Fast attack, organic decay
            if target > self.band_values[i] {
                self.band_values[i] += (target - self.band_values[i]) * (26.0 * dt).min(1.0);
            } else {
                self.band_values[i] -= (self.band_values[i] - target) * (8.5 * dt).min(1.0);
            }

            if self.band_values[i] > self.peak_values[i] {
                self.peak_values[i] = self.band_values[i];
            } else {
                self.peak_values[i] = (self.peak_values[i] - dt * 0.45).max(self.band_values[i]);
            }
        }

        // Stream time-domain PCM samples for Scope mode
        let wave_len = self.wave_buffer.len();
        let mono_len = recent_mono.len();
        if mono_len >= wave_len {
            let skip = mono_len - wave_len;
            for (i, &s) in recent_mono.iter().skip(skip).enumerate() {
                self.wave_buffer[i] = s.clamp(-1.0, 1.0);
            }
        }

        // Live stereo levels and audio metrics calculation
        let (rms_l, rms_r, peak_l, peak_r) = stereo_metrics;

        let target_l = (rms_l * 1.6).min(peak_l).clamp(0.0, 1.0);
        let target_r = (rms_r * 1.6).min(peak_r).clamp(0.0, 1.0);

        if target_l > self.left_level {
            self.left_level += (target_l - self.left_level) * (28.0 * dt).min(1.0);
        } else {
            self.left_level -= (self.left_level - target_l) * (9.0 * dt).min(1.0);
        }

        if target_r > self.right_level {
            self.right_level += (target_r - self.right_level) * (28.0 * dt).min(1.0);
        } else {
            self.right_level -= (self.right_level - target_r) * (9.0 * dt).min(1.0);
        }

        let cl_peak_l = peak_l.clamp(0.0, 1.0);
        let cl_peak_r = peak_r.clamp(0.0, 1.0);

        if cl_peak_l > self.left_peak {
            self.left_peak = cl_peak_l;
        } else {
            self.left_peak = (self.left_peak - dt * 0.4).max(self.left_level);
        }

        if cl_peak_r > self.right_peak {
            self.right_peak = cl_peak_r;
        } else {
            self.right_peak = (self.right_peak - dt * 0.4).max(self.right_level);
        }

        let raw_max_peak = peak_l.max(peak_r);
        if raw_max_peak > 0.0001 {
            self.peak_dbfs = (20.0 * raw_max_peak.log10()).clamp(-60.0, 0.0);
        } else {
            self.peak_dbfs = -60.0;
        }

        let stereo_rms = ((rms_l * rms_l + rms_r * rms_r) * 0.5).sqrt();
        if stereo_rms > 0.0001 {
            self.lufs = (20.0 * stereo_rms.log10() - 0.691).clamp(-60.0, 0.0);
        } else {
            self.lufs = -60.0;
        }

        if stereo_rms > 0.0001 && raw_max_peak > 0.0001 {
            self.dynamic_range = (20.0 * (raw_max_peak / stereo_rms).log10()).clamp(0.0, 24.0);
        } else {
            self.dynamic_range = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_initialization() {
        let tap = AudioTapBuffer::new_shared();
        let viz = Visualizer::new(VisualizerStyle::Bars, 32, tap);
        assert_eq!(viz.bands_count, 32);
        assert_eq!(viz.band_values.len(), 32);
        assert_eq!(viz.peak_values.len(), 32);
        assert_eq!(viz.wave_buffer.len(), 128);
        assert_eq!(viz.peak_dbfs, -60.0);
        assert_eq!(viz.lufs, -60.0);
        assert_eq!(viz.dynamic_range, 0.0);
        assert_eq!(viz.left_level, 0.0);
        assert_eq!(viz.right_level, 0.0);
    }

    #[test]
    fn test_visualizer_update_playback() {
        let tap = AudioTapBuffer::new_shared();
        let mut viz = Visualizer::new(VisualizerStyle::Bars, 32, tap.clone());

        // Push test PCM samples (interleaved stereo sine wave)
        let sample_rate = 44100;
        let mut pcm = Vec::with_capacity(1024);
        for i in 0..512 {
            let t = i as f32 / sample_rate as f32;
            let s_left = (t * 440.0 * std::f32::consts::TAU).sin() * 0.8;
            let s_right = (t * 880.0 * std::f32::consts::TAU).sin() * 0.6;
            pcm.push(s_left);
            pcm.push(s_right);
        }
        tap.lock().unwrap().push_interleaved(&pcm, 2, sample_rate);

        // Simulate playback frames
        for _ in 0..10 {
            viz.update(true, 1.0);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        assert!(viz.left_level > 0.0);
        assert!(viz.right_level > 0.0);
        assert!(viz.peak_dbfs > -60.0);
        assert!(viz.lufs > -60.0);
        assert!(viz.dynamic_range > 0.0);
        assert!(viz.band_values.iter().any(|&v| v > 0.0));
        assert!(viz.wave_buffer.iter().any(|&w| w.abs() > 0.0));

        // When stopped, values decay towards zero / silence
        viz.update(false, 1.0);
        assert_eq!(viz.peak_dbfs, -60.0);
        assert_eq!(viz.lufs, -60.0);
        assert_eq!(viz.dynamic_range, 0.0);
    }

    #[test]
    fn test_audio_tap_buffer_metrics() {
        let mut buffer = AudioTapBuffer::new();
        let samples = vec![0.5, -0.5, 0.8, -0.8];
        buffer.push_interleaved(&samples, 2, 44100);
        let (rms_l, rms_r, peak_l, peak_r) = buffer.compute_stereo_metrics(2);
        assert!(rms_l > 0.0);
        assert!(rms_r > 0.0);
        assert!((peak_l - 0.8).abs() < 1e-4);
        assert!((peak_r - 0.8).abs() < 1e-4);
    }
}
