use std::io::Cursor;

use porter_utils::StructWriteExt;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Allocates a new target buffer for the audio samples.
#[inline(always)]
fn allocate_target_buffer(
    audio: &Audio,
    bytes_per_sample: usize,
    bytes_per_sample_target: usize,
) -> Result<Cursor<Vec<u8>>, AudioError> {
    let data_size = (audio.data().len() / bytes_per_sample) * bytes_per_sample_target;
    let mut data: Vec<u8> = Vec::new();

    data.try_reserve_exact(data_size)?;

    Ok(Cursor::new(data))
}

/// Utility method for formats that can be coerced to another format.
pub fn software_coerce_audio(audio: &mut Audio, target: AudioFormat) -> Result<(), AudioError> {
    match (audio.format(), target) {
        (AudioFormat::IntegerPcm, AudioFormat::FloatPcm) => {
            let bytes_per_sample = audio.bits_per_sample() as usize / 8;
            let bytes_per_sample_target = 4;

            let mut data =
                allocate_target_buffer(audio, bytes_per_sample, bytes_per_sample_target)?;

            for sample in audio.data().chunks_exact(bytes_per_sample) {
                match bytes_per_sample {
                    2 => {
                        let sample = i16::from_le_bytes(sample.try_into()?);
                        let sample = sample as f32 / i16::MAX as f32;

                        data.write_struct(sample.clamp(-1.0, 1.0))?;
                    }
                    4 => {
                        let sample = i32::from_le_bytes(sample.try_into()?);
                        let sample = sample as f32 / i32::MAX as f32;

                        data.write_struct(sample.clamp(-1.0, 1.0))?;
                    }
                    _ => return Err(AudioError::ConversionError),
                }
            }

            let mut result = Audio::new(
                audio.channels(),
                audio.sample_rate(),
                32,
                AudioFormat::FloatPcm,
            )?;

            result.set_data(data.into_inner());

            *audio = result;
        }
        (AudioFormat::FloatPcm, AudioFormat::IntegerPcm) => {
            let bytes_per_sample = audio.bits_per_sample() as usize / 8;
            let bytes_per_sample_target = match bytes_per_sample {
                4 => 2,
                8 => 4,
                _ => return Err(AudioError::ConversionError),
            };

            let mut data =
                allocate_target_buffer(audio, bytes_per_sample, bytes_per_sample_target)?;

            for sample in audio.data().chunks_exact(bytes_per_sample) {
                match bytes_per_sample {
                    4 => {
                        let sample = f32::from_le_bytes(sample.try_into()?);
                        let sample = sample.clamp(-1.0, 1.0) * i16::MAX as f32;
                        let sample = sample as i16;

                        data.write_struct(sample)?;
                    }
                    8 => {
                        let sample = f64::from_le_bytes(sample.try_into()?);
                        let sample = sample.clamp(-1.0, 1.0) * i32::MAX as f64;
                        let sample = sample as i32;

                        data.write_struct(sample)?;
                    }
                    _ => return Err(AudioError::ConversionError),
                }
            }

            let mut result = Audio::new(
                audio.channels(),
                audio.sample_rate(),
                match bytes_per_sample {
                    4 => 16,
                    8 => 32,
                    _ => return Err(AudioError::ConversionError),
                },
                AudioFormat::IntegerPcm,
            )?;

            result.set_data(data.into_inner());

            *audio = result;
        }
        _ => return Err(AudioError::ConversionError),
    }

    Ok(())
}
