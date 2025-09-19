use crate::Audio;
use crate::AudioError;
use crate::AudioFormat;

/// Utility method for formats that need to be compressed during conversion.
pub fn software_compress_audio(_audio: &mut Audio, _target: AudioFormat) -> Result<(), AudioError> {
    Err(AudioError::ConversionError)
}
