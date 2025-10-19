use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use claxon::FlacReader;

use flacenc::bitsink::ByteSink;
use flacenc::component::BitRepr;
use flacenc::config::Encoder;
use flacenc::encode_with_fixed_block_size;
use flacenc::error::SourceError;
use flacenc::error::Verify;
use flacenc::source::Fill;
use flacenc::source::Source;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

struct FlacSource<'a> {
    audio: &'a Audio,
    length: u64,
    reader: Cursor<&'a [u8]>,
    buffer: Vec<u8>,
}

impl<'a> FlacSource<'a> {
    /// Constructs a new flac source from the given audio stream.
    #[inline(always)]
    pub fn new(audio: &'a Audio) -> Self {
        let length = audio.data().len() as u64;
        let reader = Cursor::new(audio.data());

        Self {
            audio,
            length,
            reader,
            buffer: Vec::new(),
        }
    }
}

/// Converts a sample of x bits to a 16bit pcm sample.
#[inline(always)]
fn convert_sample_i16(bits: u32, sample: i32) -> i16 {
    let max = (1 << (bits - 1)) - 1;

    let sample = if sample > max {
        max
    } else if sample < -max {
        -max
    } else {
        sample
    };

    let sample = sample as i64 * i16::MAX as i64 / max as i64;

    sample as i16
}

/// Converts a sample of x bits to a 32bit pcm sample.
#[inline(always)]
fn convert_sample_i32(bits: u32, sample: i32) -> i32 {
    let max = (1 << (bits - 1)) - 1;

    let sample = if sample > max {
        max
    } else if sample < -max {
        -max
    } else {
        sample
    };

    let sample = sample as i64 * i32::MAX as i64 / max as i64;

    sample as i32
}

impl Source for FlacSource<'_> {
    #[inline(always)]
    fn channels(&self) -> usize {
        self.audio.channels() as usize
    }

    #[inline(always)]
    fn bits_per_sample(&self) -> usize {
        self.audio.bits_per_sample() as usize
    }

    #[inline(always)]
    fn sample_rate(&self) -> usize {
        self.audio.sample_rate() as usize
    }

    #[inline(always)]
    fn read_samples<F: Fill>(
        &mut self,
        block_size: usize,
        dest: &mut F,
    ) -> Result<usize, SourceError> {
        if self.reader.position() == self.length {
            return Ok(0);
        }

        self.buffer.clear();

        let bytes_per_sample = self.bits_per_sample() / 8;
        let read = (block_size * bytes_per_sample * self.channels()) as u64;
        let read = read.min(self.length - self.reader.position());

        self.buffer.resize(read as usize, 0);

        let read_bytes = self
            .reader
            .read(&mut self.buffer)
            .map_err(SourceError::from_io_error)?;

        dest.fill_le_bytes(&self.buffer, bytes_per_sample)?;

        Ok(read_bytes / self.channels() / bytes_per_sample)
    }
}

/// Picks the proper format required to save the input format to a flac file type.
pub const fn pick_format(_: AudioFormat) -> AudioFormat {
    AudioFormat::IntegerPcm
}

/// Writes an audio stream to a flac file to the output stream.
pub fn to_flac<O: Write + Seek>(audio: &Audio, output: &mut O) -> Result<(), AudioError> {
    if !matches!(audio.format(), AudioFormat::IntegerPcm) {
        return Err(AudioError::UnsupportedAudioFormat(audio.format()));
    }

    let config = Encoder::default()
        .into_verified()
        .map_err(|error| error.1)?;

    let source = FlacSource::new(audio);

    let stream = encode_with_fixed_block_size(&config, source, config.block_size)?;

    let bits = stream.count_bits();
    let mut bv = ByteSink::with_capacity(bits);

    stream
        .write(&mut bv)
        .map_err(|_| AudioError::ConversionError)?;

    output.write_all(bv.as_slice())?;

    Ok(())
}

/// Reads a flac file to an audio stream.
pub fn from_flac<I: Read + Seek>(input: &mut I) -> Result<Audio, AudioError> {
    let mut reader = FlacReader::new(input)?;
    let stream_info = reader.streaminfo();

    let mut data: Vec<u8> = Vec::new();

    let bits_per_sample = if stream_info.bits_per_sample > 16 {
        32
    } else {
        16
    };

    let sample_count = stream_info.samples.unwrap_or_default();
    let sample_count_bytes = sample_count * stream_info.channels as u64 * (bits_per_sample / 8);

    data.try_reserve_exact(sample_count_bytes as _)?;

    let mut data = Cursor::new(data);

    for sample in reader.samples() {
        let sample = sample?;

        match stream_info.bits_per_sample {
            16 => data.write_all(&(sample as i16).to_le_bytes())?,
            32 => data.write_all(&sample.to_le_bytes())?,
            _ => {
                if bits_per_sample == 16 {
                    let sample = convert_sample_i16(stream_info.bits_per_sample, sample);

                    data.write_all(&sample.to_le_bytes())?;
                } else {
                    let sample = convert_sample_i32(stream_info.bits_per_sample, sample);

                    data.write_all(&sample.to_le_bytes())?;
                }
            }
        }
    }

    let mut audio = Audio::new(
        stream_info.channels,
        stream_info.sample_rate,
        bits_per_sample as u32,
        AudioFormat::IntegerPcm,
    )?;

    audio.set_data(data.into_inner());

    Ok(audio)
}
