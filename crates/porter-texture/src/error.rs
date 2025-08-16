use crate::ImageFileType;
use crate::ImageFormat;

/// Errors that can occur in the texture crate.
#[derive(Debug)]
pub enum TextureError {
    InvalidImageFormat(ImageFormat),
    InvalidDxgiFormat(u32),
    UnsupportedImageFormat(ImageFormat),
    InvalidImageSize(u32, u32),
    InvalidFrameSize(u32, u32),
    InvalidMipMaps(u32),
    FrameAllocationFailed,
    ContainerFormatInvalid(ImageFormat, ImageFileType),
    ContainerInvalid(ImageFileType),
    ConversionError,
    InvalidOperation,
    IoError(std::io::Error),
    PngEncodingError(png::EncodingError),
    PngDecodingError(png::DecodingError),
    TiffError(tiff::TiffError),
}

impl From<png::EncodingError> for TextureError {
    fn from(value: png::EncodingError) -> Self {
        Self::PngEncodingError(value)
    }
}

impl From<png::DecodingError> for TextureError {
    fn from(value: png::DecodingError) -> Self {
        Self::PngDecodingError(value)
    }
}

impl From<std::io::Error> for TextureError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<tiff::TiffError> for TextureError {
    fn from(value: tiff::TiffError) -> Self {
        Self::TiffError(value)
    }
}
