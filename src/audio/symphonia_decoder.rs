use rodio::source::SeekError;
use rodio::Source;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, SeekedTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Duration as SymphDuration, Time};
use symphonia::default::{get_codecs, get_probe};

const MAX_DECODE_RETRIES: usize = 3;

/// A seekable audio source powered by Symphonia with native support for FLAC,
/// MP3, WAV, AAC, M4A, OGG/Vorbis, and more.
///
/// Because it wraps `std::fs::File` (which implements `MediaSource` with `byte_len()`),
/// Symphonia demuxers (especially FLAC) can calculate seek tables and binary search
/// offsets properly, enabling instantaneous and sample-accurate seeking.
pub struct SymphoniaDecoderSource {
    decoder: Box<dyn Decoder>,
    format: Box<dyn FormatReader>,
    track_id: u32,
    spec: SignalSpec,
    buffer: SampleBuffer<f32>,
    current_frame_offset: usize,
    total_duration: Option<Duration>,
}

impl SymphoniaDecoderSource {
    /// Opens an audio file from a path, using extension hints when available.
    pub fn open_file(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let ext = path.extension().and_then(|e| e.to_str());
        Self::new(file, ext)
    }

    /// Creates a new decoder source wrapping an open `File`.
    pub fn new(file: File, extension: Option<&str>) -> Result<Self, String> {
        let mut hint = Hint::new();
        if let Some(ext) = extension {
            hint.with_extension(ext);
        }

        // std::fs::File implements symphonia::core::io::MediaSource with is_seekable=true
        // and byte_len=Some(metadata.len()).
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let metadata_opts = MetadataOptions::default();

        let probed = get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| format!("Failed to probe audio format: {}", e))?;

        let mut format = probed.format;

        // Select the first supported track
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| "No supported audio track found in media".to_string())?
            .clone();

        let track_id = track.id;

        let total_duration = track
            .codec_params
            .time_base
            .zip(track.codec_params.n_frames)
            .map(|(base, frames)| {
                let time = base.calc_time(frames);
                Duration::from_secs_f64(time.seconds as f64 + time.frac)
            });

        let decoder_opts = DecoderOptions::default();
        let mut decoder = get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| format!("Failed to instantiate codec decoder: {}", e))?;

        // Decode the first packet to initialize signal specs and buffer
        let mut decode_errors = 0;
        let decoded = loop {
            let packet = match format.next_packet() {
                Ok(pkt) => pkt,
                Err(Error::IoError(e)) => {
                    return Err(format!("IO error reading first packet: {}", e));
                }
                Err(e) => return Err(format!("Error reading first packet: {}", e)),
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(dec) => break dec,
                Err(Error::DecodeError(e)) => {
                    decode_errors += 1;
                    if decode_errors > MAX_DECODE_RETRIES {
                        return Err(format!("Too many decode errors reading first packet: {}", e));
                    }
                }
                Err(e) => return Err(format!("Fatal error decoding first packet: {}", e)),
            }
        };

        let spec = decoded.spec().to_owned();
        let buffer = Self::convert_to_f32_buffer(decoded, &spec);

        Ok(Self {
            decoder,
            format,
            track_id,
            spec,
            buffer,
            current_frame_offset: 0,
            total_duration,
        })
    }

    #[inline]
    fn convert_to_f32_buffer(decoded: AudioBufferRef, spec: &SignalSpec) -> SampleBuffer<f32> {
        let duration = SymphDuration::from(decoded.capacity() as u64);
        let mut buffer = SampleBuffer::<f32>::new(duration, *spec);
        buffer.copy_interleaved_ref(decoded);
        buffer
    }

    fn refine_position(&mut self, seek_res: SeekedTo) -> Result<(), rodio::source::SeekError> {
        let mut samples_to_pass = seek_res.required_ts.saturating_sub(seek_res.actual_ts);

        let packet = loop {
            let candidate = self.format.next_packet().map_err(|_| {
                SeekError::NotSupported {
                    underlying_source: "SymphoniaFormatReader::next_packet",
                }
            })?;

            if candidate.track_id() != self.track_id {
                continue;
            }

            if candidate.dur() > samples_to_pass {
                break candidate;
            } else {
                samples_to_pass -= candidate.dur();
            }
        };

        let mut decoded = self.decoder.decode(&packet);
        for _ in 0..MAX_DECODE_RETRIES {
            if decoded.is_err() {
                let packet = self.format.next_packet().map_err(|_| {
                    SeekError::NotSupported {
                        underlying_source: "SymphoniaFormatReader::retry_packet",
                    }
                })?;
                if packet.track_id() != self.track_id {
                    continue;
                }
                decoded = self.decoder.decode(&packet);
            }
        }

        let decoded = decoded.map_err(|_| SeekError::NotSupported {
            underlying_source: "SymphoniaDecoder::decode",
        })?;

        let spec = decoded.spec().to_owned();
        self.spec = spec;
        self.buffer = Self::convert_to_f32_buffer(decoded, &self.spec);
        self.current_frame_offset =
            (samples_to_pass as usize * self.spec.channels.count()).min(self.buffer.samples().len());
        Ok(())
    }
}

impl Iterator for SymphoniaDecoderSource {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_frame_offset >= self.buffer.samples().len() {
            let mut decode_errors = 0;
            let packet = loop {
                match self.format.next_packet() {
                    Ok(packet) => {
                        if packet.track_id() == self.track_id {
                            break packet;
                        }
                    }
                    Err(_) => return None,
                }
            };

            let mut current_pkt = packet;
            let decoded = loop {
                match self.decoder.decode(&current_pkt) {
                    Ok(dec) => break dec,
                    Err(Error::DecodeError(_)) => {
                        decode_errors += 1;
                        if decode_errors > MAX_DECODE_RETRIES {
                            return None;
                        }
                        current_pkt = loop {
                            match self.format.next_packet() {
                                Ok(pkt) if pkt.track_id() == self.track_id => break pkt,
                                Ok(_) => continue,
                                Err(_) => return None,
                            }
                        };
                    }
                    Err(_) => return None,
                }
            };

            if decoded.spec() != &self.spec {
                self.spec = decoded.spec().to_owned();
            }
            self.buffer = Self::convert_to_f32_buffer(decoded, &self.spec);
            self.current_frame_offset = 0;
        }

        let sample = *self.buffer.samples().get(self.current_frame_offset)?;
        self.current_frame_offset += 1;
        Some(sample)
    }
}

impl Source for SymphoniaDecoderSource {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buffer.samples().len().saturating_sub(self.current_frame_offset))
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.spec.channels.count() as u16
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let seek_beyond_end = self
            .total_duration
            .is_some_and(|dur| dur.saturating_sub(pos).as_millis() < 50);

        let time = if seek_beyond_end {
            let total = self.total_duration.unwrap_or(pos);
            let safe_secs = (total.as_secs_f64() - 0.05).max(0.0);
            Time::from(safe_secs)
        } else {
            Time::from(pos.as_secs_f64())
        };

        let to_skip = self.current_frame_offset % self.channels().max(1) as usize;

        let seek_res = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|_| SeekError::NotSupported {
                underlying_source: "SymphoniaFormatReader::seek",
            })?;

        self.decoder.reset();
        self.refine_position(seek_res)?;
        self.current_frame_offset = (self.current_frame_offset + to_skip).min(self.buffer.samples().len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symphonia_decoder_seek_flac() {
        let path = Path::new(r"C:\music\Muse\Aimyon - Marigold.flac");
        if !path.exists() {
            eprintln!("Skipping test: test file not found");
            return;
        }

        let mut source = SymphoniaDecoderSource::open_file(path).expect("Failed to open FLAC file");
        assert!(source.channels() >= 1);
        assert!(source.sample_rate() > 0);
        let total_dur = source.total_duration().expect("Duration should be present");
        assert!(total_dur.as_secs() > 60);

        // Read a few samples
        for _ in 0..100 {
            assert!(source.next().is_some());
        }

        // Try seeking to 30 seconds
        let seek_pos = Duration::from_secs(30);
        let res = source.try_seek(seek_pos);
        assert!(res.is_ok(), "Seek to 30s failed: {:?}", res.err());

        // Read samples after seek
        let sample = source.next();
        assert!(sample.is_some(), "Expected sample after seek");

        // Try seeking to 120 seconds
        let seek_pos2 = Duration::from_secs(120);
        let res2 = source.try_seek(seek_pos2);
        assert!(res2.is_ok(), "Seek to 120s failed: {:?}", res2.err());
    }

    #[test]
    fn test_symphonia_decoder_in_sink_seek() {
        let path = Path::new(r"C:\music\Muse\Aimyon - Marigold.flac");
        if !path.exists() {
            return;
        }

        let (_stream, handle) = rodio::OutputStream::try_default().ok().unzip();
        if let Some(ref h) = handle {
            let sink = rodio::Sink::try_new(h).expect("Sink::try_new failed");
            let source = SymphoniaDecoderSource::open_file(path).expect("open_file failed");
            sink.append(source);
            sink.play();

            std::thread::sleep(Duration::from_millis(50));

            let pos_before = sink.get_pos();
            assert!(pos_before > Duration::ZERO, "get_pos should advance");

            let res = sink.try_seek(Duration::from_secs(45));
            assert!(res.is_ok(), "Sink try_seek failed: {:?}", res.err());

            std::thread::sleep(Duration::from_millis(50));
            let pos_after = sink.get_pos();
            assert!(pos_after >= Duration::from_secs(45), "get_pos after seek should be >= 45s, was {:?}", pos_after);

            sink.stop();
        }
    }
}

