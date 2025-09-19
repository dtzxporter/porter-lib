use std::io::Cursor;

use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// The default first set of coeffs for MsAdpcm.
const MS_ADPCM_COEFFS1: [i16; 7] = [256, 512, 0, 192, 240, 460, 392];
/// The default second set of coeffs for MsAdpcm.
const MS_ADPCM_COEFFS2: [i16; 7] = [0, -256, 0, 64, 0, -208, -232];
/// The adaption table for MsAdpcm.
const MS_ADPCM_ADAPTION_TABLE: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MsAdpcm1ChHeader {
    predictor: u8,
    initial_delta: i16,
    sample1: i16,
    sample2: i16,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MsAdpcm2ChHeader {
    l_predictor: u8,
    r_predictor: u8,
    l_initial_delta: i16,
    r_initial_delta: i16,
    l_sample1: i16,
    r_sample1: i16,
    l_sample2: i16,
    r_sample2: i16,
}

/// Unpacks a nibble to upper, lower parts.
#[inline(always)]
fn unpack_nibble(nibble: u8) -> (u8, u8) {
    ((nibble & 0xF0) >> 4, nibble & 0xF)
}

/// Converts an unsigned to a signed nibble.
#[inline(always)]
fn signed_nibble(nibble: u8) -> i32 {
    if (nibble & 0x8) != 0 {
        (nibble as i8 - 0x10) as i32
    } else {
        (nibble as i8) as i32
    }
}

/// Clamps a i32 to a i16 value.
#[inline(always)]
fn clamp_i16(value: i32) -> i16 {
    if value.wrapping_add(0x8000) & !0xFFFF == 0 {
        value as i16
    } else {
        0x7FFF ^ value.wrapping_shr(31) as i16
    }
}

/// Decompresses one nibble and adjusts the state.
#[inline(always)]
fn decompress_nibble(
    sample1: &mut i32,
    sample2: &mut i32,
    delta: &mut i32,
    coeffs1: &[i16],
    coeffs2: &[i16],
    nibble: u8,
    predictor: usize,
) -> Result<i16, AudioError> {
    let mut state =
        ((*sample1 * coeffs1[predictor] as i32) + (*sample2 * coeffs2[predictor] as i32)) >> 8;
    state += signed_nibble(nibble) * *delta;

    *sample2 = *sample1;
    *sample1 = clamp_i16(state) as i32;

    if nibble as usize >= MS_ADPCM_ADAPTION_TABLE.len() {
        #[cfg(debug_assertions)]
        println!("MsAdpcm invalid nibble adaption index: {}", nibble);
        return Err(AudioError::ConversionError);
    }

    *delta = ((MS_ADPCM_ADAPTION_TABLE[nibble as usize] * *delta) >> 8).max(16);

    Ok(clamp_i16(state))
}

/// Decompress MsAdpcm to 16bit IntegerPcm.
pub fn decompress_ms_adpcm(audio: &mut Audio) -> Result<(), AudioError> {
    let extra = audio.extra();
    let block_align = audio.block_align().ok_or(AudioError::ConversionError)? as usize;

    let mut _samples_per_block: u16 = 0;
    let mut num_coeffs: u16 = 0;

    let mut coeffs1: Vec<i16> = Vec::new();
    let mut coeffs2: Vec<i16> = Vec::new();

    // We're doing this purely because we don't know if it's always required to specify
    // The initial coeffs in the extra block, some games may remove them and hardcode them.
    // This allows us to just adaptively decode samples using the defaults if not specified.
    if extra.len() >= 4 {
        let mut extra = Cursor::new(extra);

        _samples_per_block = extra.read_struct()?;
        num_coeffs = extra.read_struct()?;

        for _ in 0..num_coeffs {
            coeffs1.push(extra.read_struct()?);
            coeffs2.push(extra.read_struct()?);
        }
    }

    let (coeffs1, coeffs2) = if num_coeffs >= 7 {
        (&coeffs1[0..], &coeffs2[0..])
    } else {
        (&MS_ADPCM_COEFFS1[0..], &MS_ADPCM_COEFFS2[0..])
    };

    let channels = audio.channels();

    let buffer: Vec<u8> = Vec::new();
    let mut buffer = Cursor::new(buffer);

    for block in audio.data().chunks_exact(block_align) {
        let mut block = Cursor::new(block);

        if channels == 1 {
            let header: MsAdpcm1ChHeader = block.read_struct()?;
            let predictor = header.predictor as usize;

            if predictor >= coeffs1.len() || predictor >= coeffs2.len() {
                #[cfg(debug_assertions)]
                println!("MsAdpcm invalid predictor: {:#02x?}", predictor);
                return Err(AudioError::ConversionError);
            }

            buffer.write_struct(header.sample2)?;
            buffer.write_struct(header.sample1)?;

            let nibbles = block_align - size_of::<MsAdpcm1ChHeader>();

            let mut delta = header.initial_delta as i32;
            let mut sample1 = header.sample1 as i32;
            let mut sample2 = header.sample2 as i32;

            for _ in 0..nibbles {
                let nibble: u8 = block.read_struct()?;
                let (upper, lower) = unpack_nibble(nibble);

                for nibble in [upper, lower] {
                    let sample = decompress_nibble(
                        &mut sample1,
                        &mut sample2,
                        &mut delta,
                        coeffs1,
                        coeffs2,
                        nibble,
                        predictor,
                    )?;

                    buffer.write_struct(sample)?;
                }
            }
        } else if channels == 2 {
            let header: MsAdpcm2ChHeader = block.read_struct()?;
            let l_predictor = header.l_predictor as usize;
            let r_predictor = header.r_predictor as usize;

            if l_predictor >= coeffs1.len()
                || l_predictor >= coeffs2.len()
                || r_predictor >= coeffs1.len()
                || r_predictor >= coeffs2.len()
            {
                #[cfg(debug_assertions)]
                println!(
                    "MsAdpcm invalid predictors: {:#02x?} {:#02x?}",
                    l_predictor, r_predictor
                );
                return Err(AudioError::ConversionError);
            }

            buffer.write_struct(header.l_sample2)?;
            buffer.write_struct(header.r_sample2)?;
            buffer.write_struct(header.l_sample1)?;
            buffer.write_struct(header.r_sample1)?;

            let nibbles = block_align - size_of::<MsAdpcm2ChHeader>();

            let mut l_delta = header.l_initial_delta as i32;
            let mut r_delta = header.r_initial_delta as i32;
            let mut l_sample1 = header.l_sample1 as i32;
            let mut r_sample1 = header.r_sample1 as i32;
            let mut l_sample2 = header.l_sample2 as i32;
            let mut r_sample2 = header.r_sample2 as i32;

            for _ in 0..nibbles {
                let nibble: u8 = block.read_struct()?;
                let (upper, lower) = unpack_nibble(nibble);

                for (index, nibble) in [upper, lower].into_iter().enumerate() {
                    if index == 0 {
                        let sample = decompress_nibble(
                            &mut l_sample1,
                            &mut l_sample2,
                            &mut l_delta,
                            coeffs1,
                            coeffs2,
                            nibble,
                            l_predictor,
                        )?;

                        buffer.write_struct(sample)?;
                    } else {
                        let sample = decompress_nibble(
                            &mut r_sample1,
                            &mut r_sample2,
                            &mut r_delta,
                            coeffs1,
                            coeffs2,
                            nibble,
                            r_predictor,
                        )?;

                        buffer.write_struct(sample)?;
                    }
                }
            }
        } else {
            #[cfg(debug_assertions)]
            println!("MsAdpcm invalid channels: {}", channels);
            return Err(AudioError::ConversionError);
        }
    }

    let mut result = Audio::new(channels, audio.sample_rate(), 16, AudioFormat::IntegerPcm)?;

    result.set_data(buffer.into_inner());

    *audio = result;

    Ok(())
}
