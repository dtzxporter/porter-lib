use wgpu::TextureFormat;

use crate::TextureError;

/// Image formats, matches DXGI_FORMAT from DirectX.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Unknown = 0,
    R32G32B32A32Typeless,
    R32G32B32A32Float,
    R32G32B32A32Uint,
    R32G32B32A32Sint,
    R32G32B32Typeless,
    R32G32B32Float,
    R32G32B32Uint,
    R32G32B32Sint,
    R16G16B16A16Typeless,
    R16G16B16A16Float,
    R16G16B16A16Unorm,
    R16G16B16A16Uint,
    R16G16B16A16Snorm,
    R16G16B16A16Sint,
    R32G32Typeless,
    R32G32Float,
    R32G32Uint,
    R32G32Sint,
    R32G8X24Typeless,
    D32FloatS8X24Uint,
    R32FloatX8X24Typeless,
    X32TypelessG8X24Uint,
    R10G10B10A2Typeless,
    R10G10B10A2Unorm,
    R10G10B10A2Uint,
    R11G11B10Float,
    R8G8B8A8Typeless,
    R8G8B8A8Unorm,
    R8G8B8A8UnormSrgb,
    R8G8B8A8Uint,
    R8G8B8A8Snorm,
    R8G8B8A8Sint,
    R16G16Typeless,
    R16G16Float,
    R16G16Unorm,
    R16G16Uint,
    R16G16Snorm,
    R16G16Sint,
    R32Typeless,
    D32Float,
    R32Float,
    R32Uint,
    R32Sint,
    R24G8Typeless,
    D24UnormS8Uint,
    R24UnormX8Typeless,
    X24TypelessG8Uint,
    R8G8Typeless,
    R8G8Unorm,
    R8G8Uint,
    R8G8Snorm,
    R8G8Sint,
    R16Typeless,
    R16Float,
    D16Unorm,
    R16Unorm,
    R16Uint,
    R16Snorm,
    R16Sint,
    R8Typeless,
    R8Unorm,
    R8Uint,
    R8Snorm,
    R8Sint,
    A8Unorm,
    R1Unorm,
    R9G9B9E5Sharedexp,
    R8G8B8G8Unorm,
    G8R8G8B8Unorm,
    Bc1Typeless,
    Bc1Unorm,
    Bc1UnormSrgb,
    Bc2Typeless,
    Bc2Unorm,
    Bc2UnormSrgb,
    Bc3Typeless,
    Bc3Unorm,
    Bc3UnormSrgb,
    Bc4Typeless,
    Bc4Unorm,
    Bc4Snorm,
    Bc5Typeless,
    Bc5Unorm,
    Bc5Snorm,
    B5G6R5Unorm,
    B5G5R5A1Unorm,
    B8G8R8A8Unorm,
    B8G8R8X8Unorm,
    R10G10B10XrBiasA2Unorm,
    B8G8R8A8Typeless,
    B8G8R8A8UnormSrgb,
    B8G8R8X8Typeless,
    B8G8R8X8UnormSrgb,
    Bc6HTypeless,
    Bc6HUf16,
    Bc6HSf16,
    Bc7Typeless,
    Bc7Unorm,
    Bc7UnormSrgb,
    Ayuv,
    Y410,
    Y416,
    Nv12,
    P010,
    P016,
    I420Opaque,
    Yuy2,
    Y210,
    Y216,
    Nv11,
    Ai44,
    Ia44,
    P8,
    A8P8,
    B4G4R4A4Unorm,
    P208,
    V208,
    V408,

    // Added to ensure safe conversion from a raw integer value.
    Count,

    // Non-standard formats used to convert on the software side.
    R8G8B8Unorm = 0x400,
    B8G8R8Unorm,
    A8R8G8B8Unorm,
}

/// Gets whether or not an image format is palettized.
pub const fn is_format_palettized(format: ImageFormat) -> bool {
    matches!(
        format,
        ImageFormat::Ai44 | ImageFormat::Ia44 | ImageFormat::P8 | ImageFormat::A8P8
    )
}

/// Gets whether or not an image format is compressed.
pub const fn is_format_compressed(format: ImageFormat) -> bool {
    matches!(
        format,
        ImageFormat::Bc1Typeless
            | ImageFormat::Bc1Unorm
            | ImageFormat::Bc1UnormSrgb
            | ImageFormat::Bc2Typeless
            | ImageFormat::Bc2Unorm
            | ImageFormat::Bc2UnormSrgb
            | ImageFormat::Bc3Typeless
            | ImageFormat::Bc3Unorm
            | ImageFormat::Bc3UnormSrgb
            | ImageFormat::Bc4Typeless
            | ImageFormat::Bc4Unorm
            | ImageFormat::Bc4Snorm
            | ImageFormat::Bc5Typeless
            | ImageFormat::Bc5Unorm
            | ImageFormat::Bc5Snorm
            | ImageFormat::Bc6HTypeless
            | ImageFormat::Bc6HUf16
            | ImageFormat::Bc6HSf16
            | ImageFormat::Bc7Typeless
            | ImageFormat::Bc7Unorm
            | ImageFormat::Bc7UnormSrgb
    )
}

/// Gets whether or not an image format is srgb.
pub const fn is_format_srgb(format: ImageFormat) -> bool {
    matches!(
        format,
        ImageFormat::Bc1UnormSrgb
            | ImageFormat::Bc2UnormSrgb
            | ImageFormat::Bc3UnormSrgb
            | ImageFormat::Bc7UnormSrgb
            | ImageFormat::R8G8B8A8UnormSrgb
            | ImageFormat::B8G8R8A8UnormSrgb
            | ImageFormat::B8G8R8X8UnormSrgb
    )
}

/// Gets whether or not an image format is software convertable.
pub const fn is_format_requires_unpack(format: ImageFormat) -> bool {
    matches!(
        format,
        ImageFormat::R1Unorm
            | ImageFormat::R8G8B8Unorm
            | ImageFormat::B8G8R8Unorm
            | ImageFormat::R32G32B32Typeless
            | ImageFormat::R32G32B32Float
            | ImageFormat::R32G32B32Uint
            | ImageFormat::R32G32B32Sint
    )
}

/// Gets whether or not an image format and the target format are just swizzled.
pub const fn is_format_swizzled(format: ImageFormat, target: ImageFormat) -> bool {
    #[allow(clippy::match_like_matches_macro)]
    match (format, target) {
        (ImageFormat::R8G8B8Unorm, ImageFormat::B8G8R8Unorm) => true,
        (ImageFormat::B8G8R8Unorm, ImageFormat::R8G8B8Unorm) => true,
        (ImageFormat::R8G8B8A8Unorm, ImageFormat::B8G8R8A8Unorm) => true,
        (ImageFormat::R8G8B8A8Unorm, ImageFormat::A8R8G8B8Unorm) => true,
        (ImageFormat::B8G8R8A8Unorm, ImageFormat::R8G8B8A8Unorm) => true,
        (ImageFormat::B8G8R8A8Unorm, ImageFormat::A8R8G8B8Unorm) => true,
        (ImageFormat::A8R8G8B8Unorm, ImageFormat::R8G8B8A8Unorm) => true,
        (ImageFormat::A8R8G8B8Unorm, ImageFormat::B8G8R8A8Unorm) => true,
        (ImageFormat::R8G8B8A8UnormSrgb, ImageFormat::B8G8R8A8UnormSrgb) => true,
        (ImageFormat::B8G8R8A8UnormSrgb, ImageFormat::R8G8B8A8UnormSrgb) => true,
        (ImageFormat::R8G8B8A8Typeless, ImageFormat::B8G8R8A8Typeless) => true,
        (ImageFormat::B8G8R8A8Typeless, ImageFormat::R8G8B8A8Typeless) => true,
        _ => false,
    }
}

/// Gets the block dimensions for an image format.
pub const fn format_to_block_dimensions(format: ImageFormat) -> (u32, u32) {
    match format {
        // 4x4 compressed texture format.
        ImageFormat::Bc1Typeless
        | ImageFormat::Bc1Unorm
        | ImageFormat::Bc1UnormSrgb
        | ImageFormat::Bc2Typeless
        | ImageFormat::Bc2Unorm
        | ImageFormat::Bc2UnormSrgb
        | ImageFormat::Bc3Typeless
        | ImageFormat::Bc3Unorm
        | ImageFormat::Bc3UnormSrgb
        | ImageFormat::Bc4Typeless
        | ImageFormat::Bc4Unorm
        | ImageFormat::Bc4Snorm
        | ImageFormat::Bc5Typeless
        | ImageFormat::Bc5Unorm
        | ImageFormat::Bc5Snorm
        | ImageFormat::Bc6HTypeless
        | ImageFormat::Bc6HUf16
        | ImageFormat::Bc6HSf16
        | ImageFormat::Bc7Typeless
        | ImageFormat::Bc7Unorm
        | ImageFormat::Bc7UnormSrgb => (4, 4),

        // Non-compressed texture format.
        _ => (1, 1),
    }
}

/// Calculates the block size for a compressed texture only.
pub const fn format_to_block_size(format: ImageFormat) -> u32 {
    match format {
        ImageFormat::Bc1Typeless
        | ImageFormat::Bc1Unorm
        | ImageFormat::Bc1UnormSrgb
        | ImageFormat::Bc4Typeless
        | ImageFormat::Bc4Unorm
        | ImageFormat::Bc4Snorm => 8,
        ImageFormat::Bc2Typeless
        | ImageFormat::Bc2Unorm
        | ImageFormat::Bc2UnormSrgb
        | ImageFormat::Bc3Typeless
        | ImageFormat::Bc3Unorm
        | ImageFormat::Bc3UnormSrgb
        | ImageFormat::Bc5Typeless
        | ImageFormat::Bc5Unorm
        | ImageFormat::Bc5Snorm
        | ImageFormat::Bc6HTypeless
        | ImageFormat::Bc6HUf16
        | ImageFormat::Bc6HSf16
        | ImageFormat::Bc7Typeless
        | ImageFormat::Bc7Unorm
        | ImageFormat::Bc7UnormSrgb => 16,
        _ => 0,
    }
}

/// Calculates the buffer size for an image in the format, with the given width and height.
pub const fn format_to_buffer_size(format: ImageFormat, width: u32, height: u32) -> u32 {
    if is_format_compressed(format) {
        let block_size = format_to_block_size(format);
        let block_dimensions = format_to_block_dimensions(format);

        let bytes_per_row = block_size * ((width + (block_dimensions.0 - 1)) / block_dimensions.0);

        bytes_per_row * ((height + (block_dimensions.1 - 1)) / block_dimensions.1)
    } else {
        (width * height * format_to_bpp(format) + 7) / 8
    }
}

/// Converts the image format to a `wgpu` supported one if available.
pub const fn format_to_wgpu(format: ImageFormat) -> Result<TextureFormat, TextureError> {
    Ok(match format {
        // R8
        ImageFormat::R8Typeless | ImageFormat::R8Unorm => TextureFormat::R8Unorm,
        ImageFormat::R8Snorm => TextureFormat::R8Snorm,
        ImageFormat::R8Uint => TextureFormat::R8Uint,
        ImageFormat::R8Sint => TextureFormat::R8Sint,

        // R16
        ImageFormat::R16Typeless | ImageFormat::R16Unorm => TextureFormat::R16Unorm,
        ImageFormat::R16Uint => TextureFormat::R16Uint,
        ImageFormat::R16Sint => TextureFormat::R16Sint,
        ImageFormat::R16Snorm => TextureFormat::R16Snorm,
        ImageFormat::R16Float => TextureFormat::R16Float,

        // R8G8
        ImageFormat::R8G8Typeless | ImageFormat::R8G8Unorm => TextureFormat::Rg8Unorm,
        ImageFormat::R8G8Snorm => TextureFormat::Rg8Snorm,
        ImageFormat::R8G8Uint => TextureFormat::Rg8Uint,
        ImageFormat::R8G8Sint => TextureFormat::Rg8Sint,

        // R32
        ImageFormat::R32Typeless | ImageFormat::R32Uint => TextureFormat::R32Uint,
        ImageFormat::R32Sint => TextureFormat::R32Sint,
        ImageFormat::R32Float => TextureFormat::R32Float,

        // R16G16
        ImageFormat::R16G16Typeless | ImageFormat::R16G16Unorm => TextureFormat::Rg16Unorm,
        ImageFormat::R16G16Uint => TextureFormat::Rg16Uint,
        ImageFormat::R16G16Sint => TextureFormat::Rg16Sint,
        ImageFormat::R16G16Snorm => TextureFormat::Rg16Snorm,
        ImageFormat::R16G16Float => TextureFormat::Rg16Float,

        // RGBA8
        ImageFormat::R8G8B8A8Typeless | ImageFormat::R8G8B8A8Unorm => TextureFormat::Rgba8Unorm,
        ImageFormat::R8G8B8A8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        ImageFormat::R8G8B8A8Snorm => TextureFormat::Rgba8Snorm,
        ImageFormat::R8G8B8A8Uint => TextureFormat::Rgba8Uint,
        ImageFormat::R8G8B8A8Sint => TextureFormat::Rgba8Sint,

        // BGRA8
        ImageFormat::B8G8R8A8Typeless | ImageFormat::B8G8R8A8Unorm => TextureFormat::Bgra8Unorm,
        ImageFormat::B8G8R8A8UnormSrgb => TextureFormat::Bgra8UnormSrgb,

        // Packed formats
        ImageFormat::R9G9B9E5Sharedexp => TextureFormat::Rgb9e5Ufloat,
        ImageFormat::R10G10B10A2Typeless | ImageFormat::R10G10B10A2Unorm => {
            TextureFormat::Rgb10a2Unorm
        }
        ImageFormat::R11G11B10Float => TextureFormat::Rg11b10Float,

        // R32G32
        ImageFormat::R32G32Typeless | ImageFormat::R32G32Uint => TextureFormat::Rg32Uint,
        ImageFormat::R32G32Sint => TextureFormat::Rg32Sint,
        ImageFormat::R32G32Float => TextureFormat::Rg32Float,

        // R16G16B16A16
        ImageFormat::R16G16B16A16Typeless | ImageFormat::R16G16B16A16Unorm => {
            TextureFormat::Rgba16Unorm
        }
        ImageFormat::R16G16B16A16Uint => TextureFormat::Rgba16Uint,
        ImageFormat::R16G16B16A16Sint => TextureFormat::Rgba16Sint,
        ImageFormat::R16G16B16A16Snorm => TextureFormat::Rgba16Snorm,
        ImageFormat::R16G16B16A16Float => TextureFormat::Rgba16Float,

        // R32G32B32A32
        ImageFormat::R32G32B32A32Typeless | ImageFormat::R32G32B32A32Uint => {
            TextureFormat::Rgba32Uint
        }
        ImageFormat::R32G32B32A32Sint => TextureFormat::Rgba32Sint,
        ImageFormat::R32G32B32A32Float => TextureFormat::Rgba32Float,

        // Depth formats.
        ImageFormat::D16Unorm => TextureFormat::Depth16Unorm,
        ImageFormat::D24UnormS8Uint => TextureFormat::Depth24PlusStencil8,
        ImageFormat::D32Float => TextureFormat::Depth32Float,

        // BC compressed formats.
        ImageFormat::Bc1Typeless | ImageFormat::Bc1Unorm => TextureFormat::Bc1RgbaUnorm,
        ImageFormat::Bc1UnormSrgb => TextureFormat::Bc1RgbaUnormSrgb,
        ImageFormat::Bc2Typeless | ImageFormat::Bc2Unorm => TextureFormat::Bc2RgbaUnorm,
        ImageFormat::Bc2UnormSrgb => TextureFormat::Bc2RgbaUnormSrgb,
        ImageFormat::Bc3Typeless | ImageFormat::Bc3Unorm => TextureFormat::Bc3RgbaUnorm,
        ImageFormat::Bc3UnormSrgb => TextureFormat::Bc3RgbaUnormSrgb,
        ImageFormat::Bc4Typeless | ImageFormat::Bc4Unorm => TextureFormat::Bc4RUnorm,
        ImageFormat::Bc4Snorm => TextureFormat::Bc4RSnorm,
        ImageFormat::Bc5Typeless | ImageFormat::Bc5Unorm => TextureFormat::Bc5RgUnorm,
        ImageFormat::Bc5Snorm => TextureFormat::Bc5RgSnorm,
        ImageFormat::Bc6HTypeless | ImageFormat::Bc6HUf16 => TextureFormat::Bc6hRgbUfloat,
        ImageFormat::Bc6HSf16 => TextureFormat::Bc6hRgbFloat,
        ImageFormat::Bc7Typeless | ImageFormat::Bc7Unorm => TextureFormat::Bc7RgbaUnorm,
        ImageFormat::Bc7UnormSrgb => TextureFormat::Bc7RgbaUnormSrgb,

        // WGPU unsupported mapping.
        _ => return Err(TextureError::UnsupportedImageFormat(format)),
    })
}

/// Converts an unsigned to a signed image format.
pub const fn format_to_srgb(format: ImageFormat) -> ImageFormat {
    match format {
        ImageFormat::R8G8B8A8Unorm => ImageFormat::R8G8B8A8UnormSrgb,
        ImageFormat::Bc1Unorm => ImageFormat::Bc1UnormSrgb,
        ImageFormat::Bc2Unorm => ImageFormat::Bc2UnormSrgb,
        ImageFormat::Bc3Unorm => ImageFormat::Bc3UnormSrgb,
        ImageFormat::B8G8R8A8Unorm => ImageFormat::B8G8R8A8UnormSrgb,
        ImageFormat::B8G8R8X8Unorm => ImageFormat::B8G8R8X8UnormSrgb,
        ImageFormat::Bc7Unorm => ImageFormat::Bc7UnormSrgb,
        _ => format,
    }
}

/// Gets an image formats `bits` per pixel.
pub const fn format_to_bpp(format: ImageFormat) -> u32 {
    match format {
        // Unknown or unsupported format
        ImageFormat::Unknown | ImageFormat::Count => 0,

        // 1 bit per pixel
        ImageFormat::R1Unorm => 1,

        // 4 bits per pixel
        ImageFormat::Bc1Typeless
        | ImageFormat::Bc1Unorm
        | ImageFormat::Bc1UnormSrgb
        | ImageFormat::Bc4Typeless
        | ImageFormat::Bc4Unorm
        | ImageFormat::Bc4Snorm => 4,

        // 8 bits per pixel
        ImageFormat::R8Typeless
        | ImageFormat::R8Unorm
        | ImageFormat::R8Uint
        | ImageFormat::R8Snorm
        | ImageFormat::R8Sint
        | ImageFormat::A8Unorm
        | ImageFormat::Bc2Typeless
        | ImageFormat::Bc2Unorm
        | ImageFormat::Bc2UnormSrgb
        | ImageFormat::Bc3Typeless
        | ImageFormat::Bc3Unorm
        | ImageFormat::Bc3UnormSrgb
        | ImageFormat::Bc5Typeless
        | ImageFormat::Bc5Unorm
        | ImageFormat::Bc5Snorm
        | ImageFormat::Bc6HTypeless
        | ImageFormat::Bc6HUf16
        | ImageFormat::Bc6HSf16
        | ImageFormat::Bc7Typeless
        | ImageFormat::Bc7Unorm
        | ImageFormat::Bc7UnormSrgb
        | ImageFormat::Ai44
        | ImageFormat::Ia44
        | ImageFormat::P8 => 8,

        // 12 bits per pixel
        ImageFormat::Nv11 | ImageFormat::Nv12 | ImageFormat::I420Opaque => 12,

        // 16 bits per pixel
        ImageFormat::R8G8Typeless
        | ImageFormat::R8G8Unorm
        | ImageFormat::R8G8Uint
        | ImageFormat::R8G8Snorm
        | ImageFormat::R8G8Sint
        | ImageFormat::R16Typeless
        | ImageFormat::R16Float
        | ImageFormat::D16Unorm
        | ImageFormat::R16Unorm
        | ImageFormat::R16Uint
        | ImageFormat::R16Snorm
        | ImageFormat::R16Sint
        | ImageFormat::B5G6R5Unorm
        | ImageFormat::B5G5R5A1Unorm
        | ImageFormat::A8P8
        | ImageFormat::B4G4R4A4Unorm
        | ImageFormat::P208
        | ImageFormat::V208 => 16,

        // 24 bits per pixel
        ImageFormat::P010
        | ImageFormat::P016
        | ImageFormat::V408
        | ImageFormat::R8G8B8Unorm
        | ImageFormat::B8G8R8Unorm => 24,

        // 32 bits per pixel
        ImageFormat::R10G10B10A2Typeless
        | ImageFormat::R10G10B10A2Unorm
        | ImageFormat::R10G10B10A2Uint
        | ImageFormat::R11G11B10Float
        | ImageFormat::R8G8B8A8Typeless
        | ImageFormat::R8G8B8A8Unorm
        | ImageFormat::R8G8B8A8UnormSrgb
        | ImageFormat::R8G8B8A8Uint
        | ImageFormat::R8G8B8A8Snorm
        | ImageFormat::R8G8B8A8Sint
        | ImageFormat::R16G16Typeless
        | ImageFormat::R16G16Float
        | ImageFormat::R16G16Unorm
        | ImageFormat::R16G16Uint
        | ImageFormat::R16G16Snorm
        | ImageFormat::R16G16Sint
        | ImageFormat::R32Typeless
        | ImageFormat::D32Float
        | ImageFormat::R32Float
        | ImageFormat::R32Uint
        | ImageFormat::R32Sint
        | ImageFormat::R24G8Typeless
        | ImageFormat::D24UnormS8Uint
        | ImageFormat::R24UnormX8Typeless
        | ImageFormat::X24TypelessG8Uint
        | ImageFormat::R9G9B9E5Sharedexp
        | ImageFormat::R8G8B8G8Unorm
        | ImageFormat::G8R8G8B8Unorm
        | ImageFormat::B8G8R8A8Unorm
        | ImageFormat::B8G8R8X8Unorm
        | ImageFormat::R10G10B10XrBiasA2Unorm
        | ImageFormat::B8G8R8A8Typeless
        | ImageFormat::B8G8R8A8UnormSrgb
        | ImageFormat::B8G8R8X8Typeless
        | ImageFormat::B8G8R8X8UnormSrgb
        | ImageFormat::Ayuv
        | ImageFormat::Y410
        | ImageFormat::Yuy2
        | ImageFormat::A8R8G8B8Unorm => 32,

        // 64 bits per pixel
        ImageFormat::R16G16B16A16Typeless
        | ImageFormat::R16G16B16A16Float
        | ImageFormat::R16G16B16A16Unorm
        | ImageFormat::R16G16B16A16Uint
        | ImageFormat::R16G16B16A16Snorm
        | ImageFormat::R16G16B16A16Sint
        | ImageFormat::R32G32Typeless
        | ImageFormat::R32G32Float
        | ImageFormat::R32G32Uint
        | ImageFormat::R32G32Sint
        | ImageFormat::R32G8X24Typeless
        | ImageFormat::D32FloatS8X24Uint
        | ImageFormat::R32FloatX8X24Typeless
        | ImageFormat::X32TypelessG8X24Uint
        | ImageFormat::Y416
        | ImageFormat::Y210
        | ImageFormat::Y216 => 64,

        // 96 bits per pixel
        ImageFormat::R32G32B32Typeless
        | ImageFormat::R32G32B32Float
        | ImageFormat::R32G32B32Uint
        | ImageFormat::R32G32B32Sint => 96,

        // 128 bits per pixel
        ImageFormat::R32G32B32A32Typeless
        | ImageFormat::R32G32B32A32Float
        | ImageFormat::R32G32B32A32Uint
        | ImageFormat::R32G32B32A32Sint => 128,
    }
}

impl TryFrom<u32> for ImageFormat {
    type Error = TextureError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value >= ImageFormat::Unknown as u32 && value < ImageFormat::Count as u32 {
            // SAFETY: We check that the value is within the bounds of the enum, and that
            // the enum has no gaps or holes.
            #[allow(unsafe_code)]
            Ok(unsafe { std::mem::transmute::<u32, ImageFormat>(value) })
        } else {
            Err(TextureError::InvalidImageFormat(ImageFormat::Unknown))
        }
    }
}
