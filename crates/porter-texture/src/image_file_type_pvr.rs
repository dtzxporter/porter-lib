use std::io::Read;
use std::io::Seek;
use std::io::Write;

use porter_macros::assert_size;

use porter_utils::AsAligned;
use porter_utils::SeekExt;
use porter_utils::StructReadExt;

use crate::Image;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::TextureError;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PvrHeader {
    pub flags: u32,
    pub format: u32,
    pub format_ext: u32,
    pub color_space: u32,
    pub channel_type: u32,
    pub height: u32,
    pub width: u32,
    pub depth: u32,
    pub surfaces: u32,
    pub faces: u32,
    pub mip_map_count: u32,
    pub metadata_size: u32,
}

assert_size!(PvrHeader, 48);

/// Creates a proper image format from the pvr header format.
fn pvr_to_format(header: &PvrHeader) -> Result<ImageFormat, TextureError> {
    let mut format = match (header.format_ext, header.format, header.channel_type) {
        // Standard formats.
        (0, 7, _) => ImageFormat::Bc1Unorm,
        (0, 8, _) => ImageFormat::Bc2Unorm,
        (0, 9, _) => ImageFormat::Bc2Unorm,
        (0, 10, _) => ImageFormat::Bc3Unorm,
        (0, 11, _) => ImageFormat::Bc3Unorm,
        (0, 12, _) => ImageFormat::Bc4Unorm,
        (0, 13, _) => ImageFormat::Bc5Unorm,
        (0, 14, _) => ImageFormat::Bc6HUf16,
        (0, 15, _) => ImageFormat::Bc7Unorm,
        // Bit depth and channel layout formats.
        (0x08080808, 0x61626772, _) => ImageFormat::R8G8B8A8Unorm,
        (0x10101010, 0x61626772, 12 | 13) => ImageFormat::R16G16B16A16Float,
        (0x10101010, 0x61626772, _) => ImageFormat::R16G16B16A16Unorm,
        format => {
            #[cfg(debug_assertions)]
            println!("Unsupportd PVR format: {format:#02x?}");
            #[cfg(not(debug_assertions))]
            let _ = format;
            return Err(TextureError::ConversionError);
        }
    };

    if header.color_space == 1 {
        format = format.to_srgb();
    }

    // This handles unorm -> snorm:
    // Signed Byte norm.
    // Signed Byte.
    // Signed Short norm.
    // Signed Short.
    // Signed Integer norm.
    // Signed Integer.
    // Signed Float.
    if matches!(header.channel_type, 1 | 3 | 5 | 7 | 9 | 11 | 12) {
        format = format.to_snorm();
    }

    Ok(format)
}

/// Picks the proper format required to save the input format to a pvr file type.
pub const fn pick_format(format: ImageFormat) -> ImageFormat {
    format
}

/// Writes an image to a pvr file to the output stream.
pub fn to_pvr<O: Write + Seek>(_image: &Image, _output: &mut O) -> Result<(), TextureError> {
    Err(TextureError::InvalidOperation)
}

/// Reads a pvr file from the input stream to an image.
pub fn from_pvr<I: Read + Seek>(input: &mut I) -> Result<Image, TextureError> {
    let magic: u32 = input.read_struct()?;

    if magic != 0x3525650 {
        return Err(TextureError::ContainerInvalid(ImageFileType::Pvr));
    }

    let header: PvrHeader = input.read_struct()?;
    let format = pvr_to_format(&header)?;

    input.skip(header.metadata_size)?;

    let (align_w, align_h) = format.block_dimensions();

    let mut image = Image::with_mipmaps(
        header.width.as_aligned(align_w),
        header.height.as_aligned(align_h),
        header.mip_map_count.max(1),
        format,
    )?;

    for _ in 0..header.surfaces.max(header.faces) {
        image.read_frame(input)?;
    }

    Ok(image)
}
