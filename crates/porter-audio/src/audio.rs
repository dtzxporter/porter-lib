use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use porter_utils::BufferReadExt;
use porter_utils::BufferWriteExt;

use crate::AudioError;
use crate::AudioFileType;
use crate::AudioFormat;
use crate::audio_file_type_flac;
use crate::audio_file_type_wav;
use crate::software_coerce_audio;
use crate::software_compress_audio;
use crate::software_decompress_audio;

#[cfg(feature = "ogg")]
use crate::audio_file_type_ogg;

/// Represents a raw audio stream with 1-many channels.
#[derive(Debug, Clone)]
pub struct Audio {
    channels: u32,
    sample_rate: u32,
    block_align: Option<u32>,
    bits_per_sample: u32,
    format: AudioFormat,
    extra: Vec<u8>,
    data: Vec<u8>,
}

impl Audio {
    /// Creates a new audio with the given configuration and format.
    pub fn new(
        channels: u32,
        sample_rate: u32,
        bits_per_sample: u32,
        format: AudioFormat,
    ) -> Result<Self, AudioError> {
        if format == AudioFormat::Unknown {
            return Err(AudioError::InvalidAudioFormat(format));
        }

        if channels == 0 {
            return Err(AudioError::InvalidAudioChannels(channels));
        }

        if bits_per_sample == 0 {
            return Err(AudioError::InvalidAudioBitsPerSample(bits_per_sample));
        }

        Ok(Self {
            channels,
            sample_rate,
            block_align: None,
            bits_per_sample,
            format,
            extra: Vec::new(),
            data: Vec::new(),
        })
    }

    /// Creates a new audio with the given configuration, format, and block align.
    pub fn with_block_align(
        channels: u32,
        sample_rate: u32,
        block_align: u32,
        bits_per_sample: u32,
        format: AudioFormat,
    ) -> Result<Self, AudioError> {
        if format == AudioFormat::Unknown {
            return Err(AudioError::InvalidAudioFormat(format));
        }

        if channels == 0 {
            return Err(AudioError::InvalidAudioChannels(channels));
        }

        if block_align == 0 {
            return Err(AudioError::InvalidAudioBlockAlign(block_align));
        }

        if bits_per_sample == 0 {
            return Err(AudioError::InvalidAudioBitsPerSample(bits_per_sample));
        }

        Ok(Self {
            channels,
            sample_rate,
            block_align: Some(block_align),
            bits_per_sample,
            format,
            extra: Vec::new(),
            data: Vec::new(),
        })
    }

    /// Converts all channels of the audio to the specified format.
    pub fn convert(&mut self, format: AudioFormat) -> Result<(), AudioError> {
        if self.format == format {
            return Ok(());
        }

        if self.format.is_compressed() {
            software_decompress_audio(self)?;

            if self.format == format {
                return Ok(());
            }
        }

        if self.format.is_coercible(format) {
            software_coerce_audio(self, format)?;

            if self.format == format {
                return Ok(());
            }
        }

        if format.is_compressible() {
            software_compress_audio(self, format)?;

            if self.format == format {
                return Ok(());
            }
        }

        Err(AudioError::UnsupportedAudioFormat(format))
    }

    /// Calculates the optimal audio format required to save this audio stream to the given file type.
    pub fn format_for_file_type(&self, file_type: AudioFileType) -> AudioFormat {
        match file_type {
            AudioFileType::Wav => audio_file_type_wav::pick_format(self.format),
            AudioFileType::Flac => audio_file_type_flac::pick_format(self.format),
            AudioFileType::Ogg => {
                // We don't support writing these formats.
                AudioFormat::Unknown
            }
        }
    }

    /// Loads an audio stream from the given path.
    pub fn load<P: AsRef<Path>>(path: P, file_type: AudioFileType) -> Result<Self, AudioError> {
        Self::load_from(&mut File::open(path)?.buffer_read(), file_type)
    }

    /// Loads an audio stream from the given input buffer with the given file type.
    pub fn load_from<I: Read + Seek>(
        input: &mut I,
        file_type: AudioFileType,
    ) -> Result<Self, AudioError> {
        match file_type {
            AudioFileType::Wav => audio_file_type_wav::from_wav(input),
            AudioFileType::Flac => audio_file_type_flac::from_flac(input),
            #[cfg(feature = "ogg")]
            AudioFileType::Ogg => audio_file_type_ogg::from_ogg(input),
            #[cfg(not(feature = "ogg"))]
            AudioFileType::Ogg => Err(AudioError::UnsupportedAudioFileType),
        }
    }

    /// Saves the audio stream to the given file path in the given audio file type.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: AudioFileType,
    ) -> Result<(), AudioError> {
        let mut output = File::create(path)?.buffer_write();

        self.save_to(&mut output, file_type)?;

        output.flush()?;

        Ok(())
    }

    /// Saves the audio stream to the given output buffer in the given audio file type.
    pub fn save_to<O: Write + Seek>(
        &self,
        output: &mut O,
        file_type: AudioFileType,
    ) -> Result<(), AudioError> {
        match file_type {
            AudioFileType::Wav => audio_file_type_wav::to_wav(self, output),
            AudioFileType::Flac => audio_file_type_flac::to_flac(self, output),
            AudioFileType::Ogg => {
                // We don't support writing these formats.
                Err(AudioError::UnsupportedAudioFileType)
            }
        }
    }

    /// Returns the number of channels this audio stream has.
    pub fn channels(&self) -> u32 {
        self.channels
    }

    /// Returns the sample rate of the audio stream.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Returns the block alignment of the audio stream.
    pub fn block_align(&self) -> Option<u32> {
        self.block_align
    }

    /// Returns the bits per sample of the audio stream.
    pub fn bits_per_sample(&self) -> u32 {
        self.bits_per_sample
    }

    /// Returns the audio format used by the stream.
    pub fn format(&self) -> AudioFormat {
        self.format
    }

    /// Returns the extra data buffer.
    pub fn extra(&self) -> &[u8] {
        &self.extra
    }

    /// Sets a new extra data buffer.
    pub fn set_extra(&mut self, extra: Vec<u8>) {
        self.extra = extra;
    }

    /// Returns the data buffer.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Sets a new data buffer.
    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    /// Calculates the position as a duration from the given byte offset for supported formats:
    /// - [`AudioFormat::IntegerPcm`]
    /// - [`AudioFormat::FloatPcm`]
    pub fn position(&self, offset: usize) -> Result<Duration, AudioError> {
        if !matches!(self.format, AudioFormat::FloatPcm | AudioFormat::IntegerPcm) {
            return Err(AudioError::UnsupportedAudioFormat(self.format));
        }

        if offset > self.data.len() {
            return self.duration();
        }

        let block_align = self
            .block_align()
            .unwrap_or_else(|| (self.channels() * self.bits_per_sample()) / 8);

        let frame_index = offset as u64 / block_align as u64;
        let position = frame_index * 1000 / self.sample_rate() as u64;

        Ok(Duration::from_millis(position))
    }

    /// Calculates the byte offset from the given position for supported formats:
    /// - [`AudioFormat::IntegerPcm`]
    /// - [`AudioFormat::FloatPcm`]
    pub fn offset(&self, position: Duration) -> Result<usize, AudioError> {
        if !matches!(self.format, AudioFormat::FloatPcm | AudioFormat::IntegerPcm) {
            return Err(AudioError::UnsupportedAudioFormat(self.format));
        }

        let block_align = self
            .block_align()
            .unwrap_or_else(|| (self.channels() * self.bits_per_sample()) / 8);

        let num_frames = position.as_millis() as u64 * self.sample_rate() as u64 / 1000;
        let offset = num_frames * block_align as u64;

        Ok(self.data.len().min(offset as _))
    }

    /// Calculates the duration of this audio asset for supported formats:
    /// - [`AudioFormat::IntegerPcm`]
    /// - [`AudioFormat::FloatPcm`]
    pub fn duration(&self) -> Result<Duration, AudioError> {
        if !matches!(self.format, AudioFormat::FloatPcm | AudioFormat::IntegerPcm) {
            return Err(AudioError::UnsupportedAudioFormat(self.format));
        }

        let block_align = self
            .block_align()
            .unwrap_or_else(|| (self.channels() * self.bits_per_sample()) / 8);

        let num_frames = self.data.len() as u64 / block_align as u64;
        let duration = num_frames * 1000 / self.sample_rate() as u64;

        Ok(Duration::from_millis(duration))
    }
}
