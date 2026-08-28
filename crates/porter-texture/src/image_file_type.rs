use std::ffi::OsStr;

use miniserde::Deserialize;
use miniserde::Serialize;

/// Represents a supported image file type.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFileType {
    Dds,
    Png,
    Tiff,
    Tga,
    Pvr,
}

impl AsRef<OsStr> for ImageFileType {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Dds => OsStr::new("dds"),
            Self::Png => OsStr::new("png"),
            Self::Tiff => OsStr::new("tiff"),
            Self::Tga => OsStr::new("tga"),
            Self::Pvr => OsStr::new("pvr"),
        }
    }
}
