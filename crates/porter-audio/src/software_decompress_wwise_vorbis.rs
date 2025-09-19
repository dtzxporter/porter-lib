use std::io::Cursor;
use std::io::Read;
use std::io::Take;
use std::io::Write;

use lewton::audio::PreviousWindowRight;
use lewton::audio::read_audio_packet_generic;
use lewton::header::IdentHeader;
use lewton::header::SetupHeader;
use lewton::header::read_header_ident;
use lewton::header::read_header_setup;
use lewton::samples::InterleavedSamples;
use lewton::samples::Samples;

use porter_utils::BitSink;
use porter_utils::BitStream;
use porter_utils::StructReadExt;
use porter_utils::VecExt;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Packed codebook used for Wwise Vorbis.
const PACKED_CODEBOOKS_AOTUV_603: &[u8] = include_bytes!("../data/packed_codebooks_aoTuV_603.bin");

/// The setup buffer size in bits.
const SETUP_BUFFER_SIZE: usize = 4096 * 8;
/// The ident buffer size in bits.
const IDENT_BUFFER_SIZE: usize = 256;
/// The audio buffer bit size in bits.
const AUDIO_BUFFER_SIZE: usize = 16;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct WWiseVorbisExtra {
    unknown: u16,
    channel_layout: u32,
    sample_count: u32,
    setup_size: u32,
    audio_size: u32,
    unknown4: u32,
    setup_offset: u32,
    audio_offset: u32,
    unknown5: u32,
    setup_offset2: u32,
    audio_offset2: u32,
    unknown_float: f32,
    block_size_exp0: u8,
    block_size_exp1: u8,
}

/// Utility to compute `log2` without panic'ing on zero.
#[inline(always)]
fn log2(value: u32) -> u32 {
    32 - value.leading_zeros()
}

/// Utility to fetch the codebook for the given index.
fn codebook_book(index: usize) -> Result<&'static [u8], AudioError> {
    let mut codebooks = Cursor::new(PACKED_CODEBOOKS_AOTUV_603);

    codebooks.set_position(PACKED_CODEBOOKS_AOTUV_603.len() as u64 - size_of::<u32>() as u64);

    let codebook_offsets: u32 = codebooks.read_struct()?;

    codebooks.set_position(codebook_offsets as u64 + (index as u64 * size_of::<u32>() as u64));

    let offset: u32 = codebooks.read_struct()?;

    if offset as usize > PACKED_CODEBOOKS_AOTUV_603.len() {
        #[cfg(debug_assertions)]
        println!("Offset out of bounds for codebooks aoTuV: {:#02x?}", offset);
        return Err(AudioError::ConversionError);
    }

    Ok(&PACKED_CODEBOOKS_AOTUV_603[offset as usize..])
}

/// Utility to calculate the number of quants.
fn codebook_quants(dimensions: u32, entry_count: u32) -> u32 {
    let bits = log2(entry_count);
    let mut values = entry_count >> ((bits - 1) * (dimensions - 1) / dimensions);

    loop {
        let acc = values.pow(dimensions);
        let accc1 = (values + 1).pow(dimensions);

        if acc <= entry_count && accc1 > entry_count {
            return values;
        } else if acc > entry_count {
            values -= 1;
        } else {
            values += 1;
        }
    }
}

/// Utility to rebuild a codebook.
fn codebook_rebuild(setup: &mut BitSink<Cursor<Vec<u8>>>, index: usize) -> Result<(), AudioError> {
    let mut bitstream = BitStream::new_lsb(Cursor::new(codebook_book(index)?));

    setup.write_u32(24, 0x564342)?;

    let dimensions = bitstream.read_u32(4)?;
    let entry_count = bitstream.read_u32(14)?;

    setup.write_u32(16, dimensions)?;
    setup.write_u32(24, entry_count)?;

    let ordered = bitstream.read_bool()?;

    setup.write_bool(ordered)?;

    if ordered {
        let initial_length = bitstream.read_u8(5)?;

        setup.write_u8(5, initial_length)?;

        let mut current_entry = 0;

        while current_entry < entry_count {
            let bits = log2(entry_count - current_entry) as u64;
            let number = bitstream.read_u32(bits)?;

            setup.write_u32(bits, number)?;

            current_entry += number;
        }
    } else {
        let codewords_lengths = bitstream.read_u8(3)?;

        if codewords_lengths == 0 || codewords_lengths > 5 {
            #[cfg(debug_assertions)]
            println!("Invalid codewords length: {:#02x?}", codewords_lengths);
            return Err(AudioError::ConversionError);
        }

        let sparse = bitstream.read_bool()?;

        setup.write_bool(sparse)?;

        for _ in 0..entry_count {
            let present = if sparse {
                let present = bitstream.read_bool()?;

                setup.write_bool(present)?;

                present
            } else {
                true
            };

            if present {
                setup.write_u8(5, bitstream.read_u8(codewords_lengths as u64)?)?;
            }
        }
    }

    let lookup_type = bitstream.read_u8(1)?;

    setup.write_u8(4, lookup_type)?;

    if lookup_type == 1 {
        let min = bitstream.read_u32(32)?;
        let max = bitstream.read_u32(32)?;

        let value_length_minus_one = bitstream.read_u8(4)?;
        let value_length = value_length_minus_one as u64 + 1;

        let sequence_flag = bitstream.read_bool()?;

        setup.write_u32(32, min)?;
        setup.write_u32(32, max)?;
        setup.write_u8(4, value_length_minus_one)?;
        setup.write_bool(sequence_flag)?;

        for _ in 0..codebook_quants(dimensions, entry_count) {
            setup.write_u32(value_length, bitstream.read_u32(value_length)?)?;
        }
    }

    Ok(())
}

/// Utility to parse the identity packet.
fn parse_ident_packet(
    audio: &mut Audio,
    extra: &WWiseVorbisExtra,
) -> Result<IdentHeader, AudioError> {
    let mut ident = BitSink::with_capacity_lsb(IDENT_BUFFER_SIZE);

    ident.write_u8(8, 1)?;
    ident.write_all("vorbis")?;
    ident.write_u32(32, 0)?;
    ident.write_u32(8, audio.channels())?;
    ident.write_u32(32, audio.sample_rate())?;
    ident.write_u32(32, 0)?;
    ident.write_u32(32, 0)?;
    ident.write_u32(32, 0)?;
    ident.write_u8(8, extra.block_size_exp0 | (extra.block_size_exp1 << 4))?;
    ident.write_u8(8, 1)?;

    Ok(read_header_ident(&ident.into_inner()?.into_inner())?)
}

/// Utility to parse the setup packet.
fn parse_setup_packet(
    audio: &mut Audio,
    extra: &WWiseVorbisExtra,
) -> Result<(SetupHeader, Vec<bool>, u32), AudioError> {
    let mut data = Cursor::new(audio.data());

    data.set_position(extra.setup_offset as u64);

    let _packet_size: u16 = data.read_struct()?;

    let mut bitstream = BitStream::new_lsb(&mut data);
    let mut setup = BitSink::with_capacity_lsb(SETUP_BUFFER_SIZE);

    setup.write_u8(8, 0x5)?;
    setup.write_all("vorbis")?;

    let codebook_count_minus_one = bitstream.read_u16(8)?;
    let codebook_count = codebook_count_minus_one + 1;

    setup.write_u16(8, codebook_count_minus_one)?;

    for _ in 0..codebook_count {
        let index = bitstream.read_u16(10)?;

        codebook_rebuild(&mut setup, index as usize)?;
    }

    setup.write_u8(6, 0)?;
    setup.write_u16(16, 0)?;

    let floor_count_minus_one = bitstream.read_u8(6)?;
    let floor_count = floor_count_minus_one + 1;

    setup.write_u8(6, floor_count_minus_one)?;

    for _ in 0..floor_count {
        setup.write_u16(16, 0x1)?;

        let floor_partitions = bitstream.read_u8(5)?;

        setup.write_u8(5, floor_partitions)?;

        let mut floor_partition_class_list = Vec::try_with_capacity(floor_partitions as usize)?;
        let mut floor_partition_maximum_class = 0;

        for _ in 0..floor_partitions {
            let floor_partition_class = bitstream.read_u8(4)?;

            setup.write_u8(4, floor_partition_class)?;

            floor_partition_class_list.push(floor_partition_class);
            floor_partition_maximum_class =
                floor_partition_maximum_class.max(floor_partition_class);
        }

        let mut floor_class_dimensions_list =
            Vec::try_with_capacity(floor_partition_maximum_class as usize)?;

        for _ in 0..=floor_partition_maximum_class {
            let class_dimensions_minus_one = bitstream.read_u8(3)?;
            let class_subclasses = bitstream.read_u8(2)?;

            setup.write_u8(3, class_dimensions_minus_one)?;
            setup.write_u8(2, class_subclasses)?;

            if class_subclasses != 0 {
                let master_book = bitstream.read_u8(8)?;

                setup.write_u8(8, master_book)?;

                if master_book as u16 >= codebook_count {
                    #[cfg(debug_assertions)]
                    println!("Invalid floor master book: {:#02x?}", master_book);
                    return Err(AudioError::ConversionError);
                }
            }

            for _ in 0..(1 << class_subclasses) {
                let subclass_book_plus_one = bitstream.read_i16(8)?;
                let subclass_book = subclass_book_plus_one - 1;

                if subclass_book >= 0 && subclass_book >= codebook_count as i16 {
                    #[cfg(debug_assertions)]
                    println!("Invalid subclass book: {:#02x?}", subclass_book_plus_one);
                    return Err(AudioError::ConversionError);
                }

                setup.write_i16(8, subclass_book_plus_one)?;
            }

            floor_class_dimensions_list.push(class_dimensions_minus_one + 1);
        }

        let floor_multiplier_minus_one = bitstream.read_u8(2)?;
        let floor_range_bits = bitstream.read_u64(4)?;

        setup.write_u8(2, floor_multiplier_minus_one)?;
        setup.write_u64(4, floor_range_bits)?;

        for floor_partition_class in floor_partition_class_list {
            for _ in 0..floor_class_dimensions_list[floor_partition_class as usize] {
                setup.write_u16(floor_range_bits, bitstream.read_u16(floor_range_bits)?)?;
            }
        }
    }

    let residue_count_minus_one = bitstream.read_u8(6)?;
    let residue_count = residue_count_minus_one + 1;

    setup.write_u8(6, residue_count_minus_one)?;

    for _ in 0..residue_count {
        let residue_type = bitstream.read_u16(2)?;

        setup.write_u16(16, residue_type)?;

        let residue_begin = bitstream.read_u32(24)?;
        let residue_end = bitstream.read_u32(24)?;

        setup.write_u32(24, residue_begin)?;
        setup.write_u32(24, residue_end)?;

        let residue_partition_size_minus_one = bitstream.read_u32(24)?;
        let residue_classifications_minus_one = bitstream.read_u8(6)?;
        let residue_classifications = residue_classifications_minus_one + 1;

        setup.write_u32(24, residue_partition_size_minus_one)?;
        setup.write_u8(6, residue_classifications_minus_one)?;

        let residue_classbook = bitstream.read_u16(8)?;

        setup.write_u16(8, residue_classbook)?;

        if residue_classbook >= codebook_count {
            #[cfg(debug_assertions)]
            println!("Invalid residue classbook: {:#02x?}", residue_classbook);
            return Err(AudioError::ConversionError);
        }

        let mut residue_cascade = Vec::try_with_capacity(residue_classifications as usize)?;

        for _ in 0..residue_classifications {
            let low_bits = bitstream.read_u8(3)?;
            let bit_flag = bitstream.read_bool()?;

            setup.write_u8(3, low_bits)?;
            setup.write_bool(bit_flag)?;

            let high_bits = if bit_flag {
                let high_bits = bitstream.read_u8(5)?;

                setup.write_u8(5, high_bits)?;

                high_bits
            } else {
                0
            };

            residue_cascade.push(high_bits as u32 * 8 + low_bits as u32);
        }

        for residue_cascade in residue_cascade {
            for i in 0..8 {
                if (residue_cascade & (1 << i)) > 0 {
                    let residue_book = bitstream.read_u16(8)?;

                    setup.write_u16(8, residue_book)?;

                    if residue_book >= codebook_count {
                        #[cfg(debug_assertions)]
                        println!("Invalid residue book: {:#02x?}", residue_book);
                        return Err(AudioError::ConversionError);
                    }
                }
            }
        }
    }

    let mapping_count_minus_one = bitstream.read_u8(6)?;
    let mapping_count = mapping_count_minus_one + 1;

    setup.write_u8(6, mapping_count_minus_one)?;

    for _ in 0..mapping_count {
        setup.write_u16(16, 0)?;

        let submaps_flag = bitstream.read_bool()?;

        setup.write_bool(submaps_flag)?;

        let submaps = if submaps_flag {
            let submaps_minus_one = bitstream.read_u8(4)?;

            setup.write_u8(4, submaps_minus_one)?;

            submaps_minus_one + 1
        } else {
            1
        };

        let square_polar_flag = bitstream.read_bool()?;

        setup.write_bool(square_polar_flag)?;

        if square_polar_flag {
            let coupling_steps_minus_one = bitstream.read_u16(8)?;
            let coupling_steps = coupling_steps_minus_one + 1;

            setup.write_u16(8, coupling_steps_minus_one)?;

            for _ in 0..coupling_steps {
                let bits = log2(audio.channels() - 1) as u64;

                let magnitude = bitstream.read_u32(bits)?;
                let angle = bitstream.read_u32(bits)?;

                setup.write_u32(bits, magnitude)?;
                setup.write_u32(bits, angle)?;

                if magnitude >= audio.channels() || angle >= audio.channels() {
                    #[cfg(debug_assertions)]
                    println!(
                        "Invalid angle/magnitude: {:#02x?} {:#02x?}",
                        angle, magnitude
                    );
                    return Err(AudioError::ConversionError);
                }
            }
        }

        let mapping_reserved = bitstream.read_u8(2)?;

        setup.write_u8(2, mapping_reserved)?;

        if mapping_reserved != 0 {
            #[cfg(debug_assertions)]
            println!("Invalid mapping reserved: {:#02x?}", mapping_reserved);
            return Err(AudioError::ConversionError);
        }

        if submaps > 1 {
            for _ in 0..audio.channels() {
                let mapping_mux = bitstream.read_u8(4)?;

                setup.write_u8(4, mapping_mux)?;

                if mapping_mux >= submaps {
                    #[cfg(debug_assertions)]
                    println!("Invalid mapping mux: {:#02x?}", mapping_mux);
                    return Err(AudioError::ConversionError);
                }
            }
        }

        for _ in 0..submaps {
            let time_config = bitstream.read_u8(8)?;
            let floor_number = bitstream.read_u8(8)?;
            let residue_number = bitstream.read_u8(8)?;

            setup.write_u8(8, time_config)?;
            setup.write_u8(8, floor_number)?;
            setup.write_u8(8, residue_number)?;

            if floor_number >= floor_count {
                #[cfg(debug_assertions)]
                println!("Invalid floor number: {:#02x?}", floor_number);
                return Err(AudioError::ConversionError);
            }

            if residue_number >= residue_count {
                #[cfg(debug_assertions)]
                println!("Invalid residue number: {:#02x?}", residue_number);
                return Err(AudioError::ConversionError);
            }
        }
    }

    let mode_count_minus_one = bitstream.read_u8(6)?;
    let mode_count = mode_count_minus_one + 1;

    setup.write_u8(6, mode_count_minus_one)?;

    let mut mode_block_flags = Vec::try_with_capacity(mode_count as usize)?;

    for _ in 0..mode_count {
        let block_flag = bitstream.read_bool()?;

        setup.write_bool(block_flag)?;
        setup.write_u16(16, 0)?;
        setup.write_u16(16, 0)?;

        let mapping = bitstream.read_u8(8)?;

        setup.write_u8(8, mapping)?;

        if mapping >= mapping_count {
            #[cfg(debug_assertions)]
            println!("Invalid mod mapping: {:#02x?}", mapping);
            return Err(AudioError::ConversionError);
        }

        mode_block_flags.push(block_flag);
    }

    let mode_bits = log2(mode_count_minus_one as u32);

    setup.write_bool(true)?;

    let setup = read_header_setup(
        &setup.into_inner()?.into_inner(),
        audio.channels() as u8,
        (extra.block_size_exp0, extra.block_size_exp1),
    )?;

    Ok((setup, mode_block_flags, mode_bits))
}

/// Utility to parse an audio packet.
fn parse_audio_packet(
    data: &mut Take<&mut Cursor<&[u8]>>,
    packet_size: u16,
    packet_position: u64,
    mode_block_flags: &[bool],
    mode_bits: u32,
    previous_mode_block_flag: &mut bool,
) -> Result<Vec<u8>, AudioError> {
    let mut bitstream = BitStream::new_lsb(data);
    let mut audio = BitSink::with_capacity_lsb((packet_size as usize * 8) + AUDIO_BUFFER_SIZE);

    audio.write_u8(1, 0)?;

    let mode_number = bitstream.read_u8(mode_bits as u64)?;
    let remainder = bitstream.read_u8(8 - mode_bits as u64)?;

    audio.write_u8(mode_bits as u64, mode_number)?;

    let current_mode_block_flag = mode_block_flags[mode_number as usize];

    if current_mode_block_flag {
        let data = bitstream.get_ref().get_ref().get_ref();
        let next_packet = &data[packet_position as usize + packet_size as usize..];

        let next_window_flag = if next_packet.is_empty() {
            false
        } else {
            let mut next_packet = Cursor::new(next_packet);
            let packet_size: u16 = next_packet.read_struct()?;

            if packet_size > 0 {
                let next_mode_number = BitStream::new_lsb(next_packet).read_u8(mode_bits as u64)?;

                mode_block_flags[next_mode_number as usize]
            } else {
                false
            }
        };

        audio.write_bool(*previous_mode_block_flag)?;
        audio.write_bool(next_window_flag)?;
    }

    audio.write_u8(8 - mode_bits as u64, remainder)?;

    bitstream.copy(&mut audio)?;

    *previous_mode_block_flag = current_mode_block_flag;

    Ok(audio.into_inner()?.into_inner())
}

/// Decompress WwiseVorbis to 16bit IntegerPcm.
pub fn decompress_wwise_vorbis(audio: &mut Audio) -> Result<(), AudioError> {
    let extra = audio.extra();
    let mut extra = Cursor::new(extra);

    let extra: WWiseVorbisExtra = extra.read_struct()?;

    let ident_packet = parse_ident_packet(audio, &extra)?;
    let (setup_packet, mode_block_flags, mode_bits) = parse_setup_packet(audio, &extra)?;

    let mut previous_window = PreviousWindowRight::new();
    let mut previous_mode_block_flag = false;
    let mut decoded_samples = 0;

    let buffer: Vec<u8> = Vec::new();
    let mut buffer = Cursor::new(buffer);

    let mut data = Cursor::new(audio.data());
    let mut first_packet = true;

    data.set_position(extra.audio_offset as u64);

    while decoded_samples < extra.sample_count {
        let packet_size: u16 = data.read_struct()?;
        let packet_position = data.position();

        let mut packet = data.by_ref().take(packet_size as u64);

        let packet = parse_audio_packet(
            &mut packet,
            packet_size,
            packet_position,
            &mode_block_flags,
            mode_bits,
            &mut previous_mode_block_flag,
        )?;

        data.set_position(packet_position + packet_size as u64);

        let mut decoded: InterleavedSamples<i16> =
            read_audio_packet_generic(&ident_packet, &setup_packet, &packet, &mut previous_window)?;

        // For some reason the lewton library initializes the streams with the first packet.
        // But doesn't return any samples until you feed it back into it again...
        if first_packet {
            decoded = read_audio_packet_generic(
                &ident_packet,
                &setup_packet,
                &packet,
                &mut previous_window,
            )?;

            first_packet = false;
        }

        decoded_samples += decoded.num_samples() as u32;

        let truncate = if decoded_samples >= extra.sample_count {
            decoded.samples.len()
                - ((decoded_samples as usize - extra.sample_count as usize)
                    * audio.channels() as usize)
        } else {
            decoded.samples.len()
        };

        for sample in decoded.samples.into_iter().take(truncate) {
            buffer.write_all(&sample.to_le_bytes())?;
        }
    }

    let mut result = Audio::new(
        audio.channels(),
        audio.sample_rate(),
        16,
        AudioFormat::IntegerPcm,
    )?;

    result.set_data(buffer.into_inner());

    *audio = result;

    Ok(())
}
