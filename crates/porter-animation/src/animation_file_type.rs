use std::ffi::OsStr;

use bincode::Decode;
use bincode::Encode;

/// Represents a supported animation file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationFileType {
    Cast,
}

impl AsRef<OsStr> for AnimationFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Cast => OsStr::new("cast"),
        }
    }
}
