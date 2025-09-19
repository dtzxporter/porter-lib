use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Utility method for formats that require decompression before conversion.
pub fn software_decompress_audio(audio: &mut Audio) -> Result<(), AudioError> {
    match audio.format() {
        AudioFormat::MsAdpcm => {
            #[cfg(feature = "ms-adpcm")]
            crate::decompress_ms_adpcm(audio)?;
            #[cfg(not(feature = "ms-adpcm"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        AudioFormat::WwiseVorbis => {
            #[cfg(feature = "wwise-vorbis")]
            crate::decompress_wwise_vorbis(audio)?;
            #[cfg(not(feature = "wwise-vorbis"))]
            return Err(AudioError::ConversionFeatureDisabled);
        }
        _ => return Err(AudioError::ConversionError),
    }

    #[allow(unreachable_code)]
    Ok(())
}
