use lewton::inside_ogg::OggStreamReader;

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Reads a flac file to an audio stream.
pub fn from_ogg<I: Read + Seek>(input: &mut I) -> Result<Audio, AudioError> {
    let mut reader = OggStreamReader::new(input)?;
    let mut data: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    while let Some(samples) = reader.read_dec_packet_itl()? {
        for sample in samples {
            data.write_all(&sample.to_le_bytes())?;
        }
    }

    let mut audio = Audio::new(
        reader.ident_hdr.audio_channels as u32,
        reader.ident_hdr.audio_sample_rate,
        16,
        AudioFormat::IntegerPcm,
    )?;

    audio.set_data(data.into_inner());

    Ok(audio)
}
