use bincode::Decode;
use bincode::Encode;

/// Represents a supported audio file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFileType {
    Wav,
    Flac,
}
