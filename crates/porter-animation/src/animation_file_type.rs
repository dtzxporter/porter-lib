use std::ffi::OsStr;

use miniserde::Deserialize;
use miniserde::Serialize;

/// Represents a supported animation file type.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
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
