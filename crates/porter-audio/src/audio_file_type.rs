use std::ffi::OsStr;

use miniserde::Deserialize;
use miniserde::Serialize;

/// Represents a supported audio file type.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFileType {
    Wav,
    Flac,
    Ogg,
}

impl AsRef<OsStr> for AudioFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Wav => OsStr::new("wav"),
            Self::Flac => OsStr::new("flac"),
            Self::Ogg => OsStr::new("ogg"),
        }
    }
}
