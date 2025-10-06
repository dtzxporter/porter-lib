use std::io::Cursor;
use std::io::Write;

use porter_utils::StructWriteExt;

use crate::Audio;
use crate::AudioError;
use crate::AudioFileType;

/// Flags for the metadata block.
const BLOCK_FLAGS: u8 = 0x80;
/// Size in bytes of the metadata block.
const BLOCK_SIZE: u64 = 0x22;

/// Minimum block size in samples.
const MIN_BLOCK_SIZE: u16 = 0x400;
/// Maximum block size in samples.
const MAX_BLOCK_SIZE: u16 = 0x1000;

/// Minimum frame size in bytes.
const MIN_FRAME_SIZE: u64 = 0x0;
/// Maximum frame size in bytes.
const MAX_FRAME_SIZE: u64 = 0x0;

/// Decompress raw flac stream without headers to 8-32bit IntegerPcm.
pub fn decompress_raw_flac(audio: &mut Audio) -> Result<(), AudioError> {
    let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    buffer.write_all(b"fLaC")?;

    buffer.write_struct(BLOCK_FLAGS)?;

    buffer.write_be_sized_integer(BLOCK_SIZE, 3)?;

    buffer.write_struct(MIN_BLOCK_SIZE.swap_bytes())?;
    buffer.write_struct(MAX_BLOCK_SIZE.swap_bytes())?;

    buffer.write_be_sized_integer(MIN_FRAME_SIZE, 3)?;
    buffer.write_be_sized_integer(MAX_FRAME_SIZE, 3)?;

    let sample_rate = audio.sample_rate();
    let channels = audio.channels();
    let bits_per_sample = audio.bits_per_sample();

    #[allow(clippy::identity_op)]
    let flags: u64 = ((sample_rate as u64) << 44)
        | ((channels as u64 - 1) << 41)
        | ((bits_per_sample as u64 - 1) << 36)
        // We do not expose frame count in audio because it's not necessary to
        // decode any known audio format, and we don't expose extra fields for flac.
        | 0x0u64;

    buffer.write_struct(flags.swap_bytes())?;

    buffer.write_all(&[0; 16])?;

    std::io::copy(&mut audio.data(), &mut buffer)?;

    buffer.set_position(0);

    *audio = Audio::load_from(&mut buffer, AudioFileType::Flac)?;

    Ok(())
}
