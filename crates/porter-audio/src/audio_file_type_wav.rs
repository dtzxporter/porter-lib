use std::io::Read;
use std::io::Seek;
use std::io::Write;

use porter_utils::SeekExt;
use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Audio;
use crate::AudioError;
use crate::AudioFileType;
use crate::AudioFormat;

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
struct WavefmtHeader {
    size: u32,
    format: u16,
    channel_count: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct WavefmtHeaderExtra {
    size: u32,
    format: u16,
    channel_count: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
    extra_size: u16,
}

/// Calculates the average bytes per second.
const fn compute_byte_rate(bits_per_sample: u32, channels: u32, sample_rate: u32) -> u32 {
    (sample_rate * bits_per_sample * channels) / 8
}

/// Picks the proper format required to save the input format to a wav file type.
pub const fn pick_format(format: AudioFormat) -> AudioFormat {
    format
}

/// Writes an audio stream to a wav file to the output stream.
pub fn to_wav<O: Write + Seek>(audio: &Audio, output: &mut O) -> Result<(), AudioError> {
    output.write_struct(0x46464952u32)?; // 'RIFF'

    let file_size_offset = output.stream_position()?;

    output.write_struct(0x0u32)?;
    output.write_struct(0x20746D6645564157u64)?; // 'WAVEfmt '

    let sample_rate = audio.sample_rate();
    let bits_per_sample = audio.bits_per_sample();
    let channels = audio.channels();
    let data = audio.data();
    let extra = audio.extra();
    let byte_rate = compute_byte_rate(bits_per_sample, channels, sample_rate);

    match audio.format() {
        AudioFormat::IntegerPcm => {
            let header = WavefmtHeader {
                size: size_of::<WavefmtHeader>() as u32 - size_of::<u32>() as u32,
                format: 0x1,
                channel_count: channels as u16,
                sample_rate,
                byte_rate,
                block_align: ((bits_per_sample * channels) / 8) as u16,
                bits_per_sample: bits_per_sample as u16,
            };

            output.write_struct(header)?;
        }
        AudioFormat::MsAdpcm => {
            let extra_size = extra.len() as u16;
            let block_align = audio.block_align().ok_or(AudioError::ConversionError)? as u16;

            let header = WavefmtHeaderExtra {
                size: size_of::<WavefmtHeaderExtra>() as u32 - size_of::<u32>() as u32
                    + extra_size as u32,
                format: 0x2,
                channel_count: channels as u16,
                sample_rate,
                byte_rate,
                block_align,
                bits_per_sample: bits_per_sample as u16,
                extra_size,
            };

            output.write_struct(header)?;
        }
        AudioFormat::FloatPcm => {
            let header = WavefmtHeader {
                size: size_of::<WavefmtHeader>() as u32 - size_of::<u32>() as u32,
                format: 0x3,
                channel_count: channels as u16,
                sample_rate,
                byte_rate,
                block_align: ((bits_per_sample * channels) / 8) as u16,
                bits_per_sample: bits_per_sample as u16,
            };

            output.write_struct(header)?;
        }
        AudioFormat::WwiseVorbis => {
            let extra_size = extra.len() as u16;

            let header = WavefmtHeaderExtra {
                size: size_of::<WavefmtHeaderExtra>() as u32 - size_of::<u32>() as u32
                    + extra_size as u32,
                format: 0xFFFF,
                channel_count: channels as u16,
                sample_rate,
                byte_rate,
                block_align: 0,
                bits_per_sample: 0,
                extra_size,
            };

            output.write_struct(header)?;
        }
        _ => return Err(AudioError::UnsupportedAudioFormat(audio.format())),
    };

    output.write_all(extra)?;

    output.write_struct(0x61746164u32)?; // 'data'
    output.write_struct(data.len() as u32)?;

    output.write_all(data)?;

    let file_end_offset = output.stream_position()?;
    let file_size: u32 =
        file_end_offset as u32 - (file_size_offset + size_of::<u32>() as u64) as u32;

    output.reset_to(file_size_offset)?;
    output.write_struct(file_size)?;

    output.reset_to(file_end_offset)?;

    Ok(())
}

/// Reads a wav file to an audio stream.
pub fn from_wav<I: Read + Seek>(input: &mut I) -> Result<Audio, AudioError> {
    let magic: u32 = input.read_struct()?;

    // 'RIFF'
    if magic != 0x46464952 {
        return Err(AudioError::ContainerInvalid(AudioFileType::Wav));
    }

    let _file_size: u32 = input.read_struct()?;

    let mut data = Vec::new();
    let mut extra = Vec::new();
    let mut header: WavefmtHeader = Default::default();

    loop {
        let block: u32 = input.read_struct()?;
        let size: u32 = input.read_struct()?;

        match block {
            // 'WAVE'
            0x45564157 => {
                // 'fmt '
                if size != 0x20746D66 {
                    return Err(AudioError::ContainerInvalid(AudioFileType::Wav));
                }

                header = input.read_struct()?;

                if header.size >= 0x12 {
                    let size: u16 = input.read_struct()?;

                    extra.try_reserve_exact(size as usize)?;
                    extra.resize(size as usize, 0);

                    input.read_exact(&mut extra)?;
                }
            }
            // 'data'
            0x61746164 => {
                data.try_reserve_exact(size as usize)?;
                data.resize(size as usize, 0);

                input.read_exact(&mut data)?;
                break;
            }
            _ => {
                #[cfg(all(debug_assertions, feature = "debug"))]
                {
                    // Ignored blocks:
                    // ['cue ', 'LIST', 'smpl', 'JUNK', 'akd ']
                    if ![0x20657563, 0x5453494C, 0x6C706D73, 0x4B4E554A, 0x20646B61]
                        .contains(&block)
                    {
                        println!(
                            "Skipping block: {:#02X?} '{}{}{}{}' @ {:#02X?}",
                            block,
                            (block & 0xFF) as u8 as char,
                            ((block >> 8) & 0xFF) as u8 as char,
                            ((block >> 16) & 0xFF) as u8 as char,
                            ((block >> 24) & 0xFF) as u8 as char,
                            input.stream_position()? - 8
                        );
                    }
                }

                input.skip(size)?;
            }
        }
    }

    let format = match header.format {
        0x1 => AudioFormat::IntegerPcm,
        0x2 => AudioFormat::MsAdpcm,
        0x3 => AudioFormat::FloatPcm,
        0xFFFF => {
            header.block_align = 1;
            header.bits_per_sample = 8;

            AudioFormat::WwiseVorbis
        }
        _ => {
            #[cfg(debug_assertions)]
            println!("Unknown wav format: {:#02x?}", { header.format });
            return Err(AudioError::ContainerFormatInvalid(
                AudioFormat::Unknown,
                AudioFileType::Wav,
            ));
        }
    };

    let mut audio = Audio::with_block_align(
        header.channel_count as u32,
        header.sample_rate,
        header.block_align as u32,
        header.bits_per_sample as u32,
        format,
    )?;

    audio.set_data(data);
    audio.set_extra(extra);

    Ok(audio)
}
