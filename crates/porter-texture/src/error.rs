use crate::ImageFileType;
use crate::ImageFormat;

#[derive(Debug)]
pub enum TextureError {
    InvalidImageFormat(ImageFormat),
    UnsupportedImageFormat(ImageFormat),
    InvalidImageSize(u32, u32),
    InvalidFrameSize(u32, u32),
    FrameAllocationFailed,
    ContainerFormatInvalid(ImageFormat, ImageFileType),
    ConversionError,
    IoError(std::io::Error),
    PngEncodingError(png::EncodingError),
    TiffError(tiff::TiffError),
}

impl From<png::EncodingError> for TextureError {
    fn from(value: png::EncodingError) -> Self {
        Self::PngEncodingError(value)
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
