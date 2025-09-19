use crate::AudioFileType;
use crate::AudioFormat;

/// Errors that can occor in the audio crate.
#[derive(Debug)]
pub enum AudioError {
    UnsupportedAudioFormat(AudioFormat),
    UnsupportedAudioFileType,
    InvalidAudioFormat(AudioFormat),
    InvalidAudioChannels(u32),
    InvalidAudioBlockAlign(u32),
    InvalidAudioBitsPerSample(u32),
    ContainerFormatInvalid(AudioFormat, AudioFileType),
    ContainerInvalid(AudioFileType),
    ConversionError,
    ConversionFeatureDisabled,
    IoError(std::io::Error),
    FlacVerifyError(flacenc::error::VerifyError),
    FlacSourceError,
    FlacEncodeError,
    FlacDecodeError(claxon::Error),
    TryFromSliceError(std::array::TryFromSliceError),
    TryReserveError(std::collections::TryReserveError),
    #[cfg(feature = "wwise-vorbis")]
    WwiseHeaderReadError(lewton::header::HeaderReadError),
    #[cfg(feature = "wwise-vorbis")]
    WwiseAudioReadError(lewton::audio::AudioReadError),
    #[cfg(feature = "ogg")]
    OggVorbisError(lewton::VorbisError),
    #[cfg(feature = "ogg")]
    OggReadError(lewton::OggReadError),
}

impl From<std::io::Error> for AudioError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<flacenc::error::VerifyError> for AudioError {
    fn from(value: flacenc::error::VerifyError) -> Self {
        Self::FlacVerifyError(value)
    }
}

impl From<flacenc::error::SourceError> for AudioError {
    fn from(_: flacenc::error::SourceError) -> Self {
        Self::FlacSourceError
    }
}

impl From<flacenc::error::EncodeError> for AudioError {
    fn from(_: flacenc::error::EncodeError) -> Self {
        Self::FlacEncodeError
    }
}

impl From<claxon::Error> for AudioError {
    fn from(value: claxon::Error) -> Self {
        Self::FlacDecodeError(value)
    }
}

impl From<std::array::TryFromSliceError> for AudioError {
    fn from(value: std::array::TryFromSliceError) -> Self {
        Self::TryFromSliceError(value)
    }
}

impl From<std::collections::TryReserveError> for AudioError {
    fn from(value: std::collections::TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}

#[cfg(feature = "wwise-vorbis")]
impl From<lewton::header::HeaderReadError> for AudioError {
    fn from(value: lewton::header::HeaderReadError) -> Self {
        Self::WwiseHeaderReadError(value)
    }
}

#[cfg(feature = "wwise-vorbis")]
impl From<lewton::audio::AudioReadError> for AudioError {
    fn from(value: lewton::audio::AudioReadError) -> Self {
        Self::WwiseAudioReadError(value)
    }
}

#[cfg(feature = "ogg")]
impl From<lewton::VorbisError> for AudioError {
    fn from(value: lewton::VorbisError) -> Self {
        Self::OggVorbisError(value)
    }
}

#[cfg(feature = "ogg")]
impl From<lewton::OggReadError> for AudioError {
    fn from(value: lewton::OggReadError) -> Self {
        Self::OggReadError(value)
    }
}
