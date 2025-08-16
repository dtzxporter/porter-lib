use std::ffi::OsStr;

use bincode::Decode;
use bincode::Encode;

/// Represents a supported image file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFileType {
    Dds,
    Png,
    Tiff,
    Tga,
}

impl AsRef<OsStr> for ImageFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Dds => OsStr::new("dds"),
            Self::Png => OsStr::new("png"),
            Self::Tiff => OsStr::new("tiff"),
            Self::Tga => OsStr::new("tga"),
        }
    }
}
