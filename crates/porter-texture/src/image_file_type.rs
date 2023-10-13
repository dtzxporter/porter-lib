use std::ffi::OsStr;

use bincode::Decode;
use bincode::Encode;

/// Represents a supported image file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFileType {
    Dds,
    Png,
    Tiff,
}

impl AsRef<OsStr> for ImageFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            ImageFileType::Dds => OsStr::new("dds"),
            ImageFileType::Png => OsStr::new("png"),
            ImageFileType::Tiff => OsStr::new("tiff"),
        }
    }
}
