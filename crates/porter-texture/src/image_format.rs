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

impl ImageFormat {
    /// Calculates the size in bytes of a buffer that will fit the image format of the given width and height.
    pub const fn buffer_size(&self, width: u32, height: u32) -> u32 {
        if self.is_compressed() {
            let block_size = self.block_size();
            let (block_x, block_y) = self.block_dimensions();

            (block_size * width.div_ceil(block_x)) * height.div_ceil(block_y)
        } else {
            let width = width as usize;
            let height = height as usize;
            let bits_per_pixel = self.bits_per_pixel() as usize;

            let bits_size = width * height * bits_per_pixel;
            let byte_size = bits_size.div_ceil(8);

            debug_assert!(byte_size <= u32::MAX as usize);

            byte_size as u32
        }
    }

    /// Gets the bits per pixel for the image format.
    pub const fn bits_per_pixel(&self) -> u32 {
        match self {
            // Unknown or unsupported format
            Self::Unknown | Self::Count => 0,

            // 1 bit per pixel
            Self::R1Unorm => 1,

            // 4 bits per pixel
            Self::Bc1Typeless
            | Self::Bc1Unorm
            | Self::Bc1UnormSrgb
            | Self::Bc4Typeless
            | Self::Bc4Unorm
            | Self::Bc4Snorm => 4,

            // 8 bits per pixel
            Self::R8Typeless
            | Self::R8Unorm
            | Self::R8Uint
            | Self::R8Snorm
            | Self::R8Sint
            | Self::A8Unorm
            | Self::Bc2Typeless
            | Self::Bc2Unorm
            | Self::Bc2UnormSrgb
            | Self::Bc3Typeless
            | Self::Bc3Unorm
            | Self::Bc3UnormSrgb
            | Self::Bc5Typeless
            | Self::Bc5Unorm
            | Self::Bc5Snorm
            | Self::Bc6HTypeless
            | Self::Bc6HUf16
            | Self::Bc6HSf16
            | Self::Bc7Typeless
            | Self::Bc7Unorm
            | Self::Bc7UnormSrgb
            | Self::Ai44
            | Self::Ia44
            | Self::P8 => 8,

            // 12 bits per pixel
            Self::Nv11 | Self::Nv12 | Self::I420Opaque => 12,

            // 16 bits per pixel
            Self::R8G8Typeless
            | Self::R8G8Unorm
            | Self::R8G8Uint
            | Self::R8G8Snorm
            | Self::R8G8Sint
            | Self::R16Typeless
            | Self::R16Float
            | Self::D16Unorm
            | Self::R16Unorm
            | Self::R16Uint
            | Self::R16Snorm
            | Self::R16Sint
            | Self::B5G6R5Unorm
            | Self::B5G5R5A1Unorm
            | Self::A8P8
            | Self::B4G4R4A4Unorm
            | Self::P208
            | Self::V208 => 16,

            // 24 bits per pixel
            Self::P010 | Self::P016 | Self::V408 | Self::R8G8B8Unorm | Self::B8G8R8Unorm => 24,

            // 32 bits per pixel
            Self::R10G10B10A2Typeless
            | Self::R10G10B10A2Unorm
            | Self::R10G10B10A2Uint
            | Self::R11G11B10Float
            | Self::R8G8B8A8Typeless
            | Self::R8G8B8A8Unorm
            | Self::R8G8B8A8UnormSrgb
            | Self::R8G8B8A8Uint
            | Self::R8G8B8A8Snorm
            | Self::R8G8B8A8Sint
            | Self::R16G16Typeless
            | Self::R16G16Float
            | Self::R16G16Unorm
            | Self::R16G16Uint
            | Self::R16G16Snorm
            | Self::R16G16Sint
            | Self::R32Typeless
            | Self::D32Float
            | Self::R32Float
            | Self::R32Uint
            | Self::R32Sint
            | Self::R24G8Typeless
            | Self::D24UnormS8Uint
            | Self::R24UnormX8Typeless
            | Self::X24TypelessG8Uint
            | Self::R9G9B9E5Sharedexp
            | Self::R8G8B8G8Unorm
            | Self::G8R8G8B8Unorm
            | Self::B8G8R8A8Unorm
            | Self::B8G8R8X8Unorm
            | Self::R10G10B10XrBiasA2Unorm
            | Self::B8G8R8A8Typeless
            | Self::B8G8R8A8UnormSrgb
            | Self::B8G8R8X8Typeless
            | Self::B8G8R8X8UnormSrgb
            | Self::Ayuv
            | Self::Y410
            | Self::Yuy2
            | Self::A8R8G8B8Unorm => 32,

            // 64 bits per pixel
            Self::R16G16B16A16Typeless
            | Self::R16G16B16A16Float
            | Self::R16G16B16A16Unorm
            | Self::R16G16B16A16Uint
            | Self::R16G16B16A16Snorm
            | Self::R16G16B16A16Sint
            | Self::R32G32Typeless
            | Self::R32G32Float
            | Self::R32G32Uint
            | Self::R32G32Sint
            | Self::R32G8X24Typeless
            | Self::D32FloatS8X24Uint
            | Self::R32FloatX8X24Typeless
            | Self::X32TypelessG8X24Uint
            | Self::Y416
            | Self::Y210
            | Self::Y216 => 64,

            // 96 bits per pixel
            Self::R32G32B32Typeless
            | Self::R32G32B32Float
            | Self::R32G32B32Uint
            | Self::R32G32B32Sint => 96,

            // 128 bits per pixel
            Self::R32G32B32A32Typeless
            | Self::R32G32B32A32Float
            | Self::R32G32B32A32Uint
            | Self::R32G32B32A32Sint => 128,
        }
    }

    /// Calculates the block dimensions for a compressed image format.
    pub const fn block_dimensions(&self) -> (u32, u32) {
        match self {
            // 4x4 compressed texture format.
            Self::Bc1Typeless
            | Self::Bc1Unorm
            | Self::Bc1UnormSrgb
            | Self::Bc2Typeless
            | Self::Bc2Unorm
            | Self::Bc2UnormSrgb
            | Self::Bc3Typeless
            | Self::Bc3Unorm
            | Self::Bc3UnormSrgb
            | Self::Bc4Typeless
            | Self::Bc4Unorm
            | Self::Bc4Snorm
            | Self::Bc5Typeless
            | Self::Bc5Unorm
            | Self::Bc5Snorm
            | Self::Bc6HTypeless
            | Self::Bc6HUf16
            | Self::Bc6HSf16
            | Self::Bc7Typeless
            | Self::Bc7Unorm
            | Self::Bc7UnormSrgb => (4, 4),

            // Non-compressed texture format.
            _ => (1, 1),
        }
    }

    /// Calculates the block size for a compressed image format.
    pub const fn block_size(&self) -> u32 {
        match self {
            // 8 bytes per block.
            Self::Bc1Typeless
            | Self::Bc1Unorm
            | Self::Bc1UnormSrgb
            | Self::Bc4Typeless
            | Self::Bc4Unorm
            | Self::Bc4Snorm => 8,

            // 16 bytes per block.
            Self::Bc2Typeless
            | Self::Bc2Unorm
            | Self::Bc2UnormSrgb
            | Self::Bc3Typeless
            | Self::Bc3Unorm
            | Self::Bc3UnormSrgb
            | Self::Bc5Typeless
            | Self::Bc5Unorm
            | Self::Bc5Snorm
            | Self::Bc6HTypeless
            | Self::Bc6HUf16
            | Self::Bc6HSf16
            | Self::Bc7Typeless
            | Self::Bc7Unorm
            | Self::Bc7UnormSrgb => 16,

            // Non-compressed pixel format.
            _ => 0,
        }
    }

    /// Whether or not the image format is palettized.
    pub const fn is_palettized(&self) -> bool {
        matches!(self, Self::Ai44 | Self::Ia44 | Self::P8 | Self::A8P8)
    }

    // Whether or not the image format is compressed.
    pub const fn is_compressed(&self) -> bool {
        matches!(
            self,
            Self::Bc1Typeless
                | Self::Bc1Unorm
                | Self::Bc1UnormSrgb
                | Self::Bc2Typeless
                | Self::Bc2Unorm
                | Self::Bc2UnormSrgb
                | Self::Bc3Typeless
                | Self::Bc3Unorm
                | Self::Bc3UnormSrgb
                | Self::Bc4Typeless
                | Self::Bc4Unorm
                | Self::Bc4Snorm
                | Self::Bc5Typeless
                | Self::Bc5Unorm
                | Self::Bc5Snorm
                | Self::Bc6HTypeless
                | Self::Bc6HUf16
                | Self::Bc6HSf16
                | Self::Bc7Typeless
                | Self::Bc7Unorm
                | Self::Bc7UnormSrgb
        )
    }

    /// Whether or not the image format is in sRGB colorspace.
    pub const fn is_srgb(&self) -> bool {
        matches!(
            self,
            Self::Bc1UnormSrgb
                | Self::Bc2UnormSrgb
                | Self::Bc3UnormSrgb
                | Self::Bc7UnormSrgb
                | Self::R8G8B8A8UnormSrgb
                | Self::B8G8R8A8UnormSrgb
                | Self::B8G8R8X8UnormSrgb
        )
    }

    /// Whether or not the image format is software convertible.
    pub const fn is_unpack_required(&self) -> bool {
        matches!(
            self,
            Self::R1Unorm
                | Self::R8G8B8Unorm
                | Self::B8G8R8Unorm
                | Self::R32G32B32Typeless
                | Self::R32G32B32Float
                | Self::R32G32B32Uint
                | Self::R32G32B32Sint
        )
    }

    /// Whether or not the image format is a swizzled version of the given format.
    pub const fn is_swizzled(&self, format: Self) -> bool {
        matches!(
            (self, format),
            (Self::R8G8B8Unorm, Self::B8G8R8Unorm)
                | (Self::B8G8R8Unorm, Self::R8G8B8Unorm)
                | (Self::R8G8B8A8Unorm, Self::B8G8R8A8Unorm)
                | (Self::R8G8B8A8Unorm, Self::A8R8G8B8Unorm)
                | (Self::B8G8R8A8Unorm, Self::R8G8B8A8Unorm)
                | (Self::B8G8R8A8Unorm, Self::A8R8G8B8Unorm)
                | (Self::A8R8G8B8Unorm, Self::R8G8B8A8Unorm)
                | (Self::A8R8G8B8Unorm, Self::B8G8R8A8Unorm)
                | (Self::R8G8B8A8UnormSrgb, Self::B8G8R8A8UnormSrgb)
                | (Self::B8G8R8A8UnormSrgb, Self::R8G8B8A8UnormSrgb)
                | (Self::R8G8B8A8Typeless, Self::B8G8R8A8Typeless)
                | (Self::B8G8R8A8Typeless, Self::R8G8B8A8Typeless)
        )
    }

    /// Whether or not the image format can be resized.
    pub const fn is_resizable(&self) -> bool {
        matches!(
            self,
            Self::R8G8B8A8Typeless
                | Self::R8G8B8A8Unorm
                | Self::R8G8B8A8UnormSrgb
                | Self::R8G8B8A8Uint
                | Self::R8G8B8A8Snorm
                | Self::R8G8B8A8Sint
                | Self::B8G8R8A8Unorm
                | Self::B8G8R8A8UnormSrgb
                | Self::B8G8R8A8Typeless
                | Self::A8R8G8B8Unorm
        )
    }

    /// Returns the sRGB colorspace version of this image format if one exists, otherwise returns itself.
    pub const fn to_srgb(&self) -> Self {
        match self {
            Self::R8G8B8A8Unorm => Self::R8G8B8A8UnormSrgb,
            Self::Bc1Unorm => Self::Bc1UnormSrgb,
            Self::Bc2Unorm => Self::Bc2UnormSrgb,
            Self::Bc3Unorm => Self::Bc3UnormSrgb,
            Self::B8G8R8A8Unorm => Self::B8G8R8A8UnormSrgb,
            Self::B8G8R8X8Unorm => Self::B8G8R8X8UnormSrgb,
            Self::Bc7Unorm => Self::Bc7UnormSrgb,
            _ => *self,
        }
    }

    /// Returns the wgpu version of this image format if one exists, otherwise returns an error.
    pub const fn to_wgpu(&self) -> Result<TextureFormat, TextureError> {
        Ok(match self {
            // R8
            Self::R8Typeless | Self::R8Unorm => TextureFormat::R8Unorm,
            Self::R8Snorm => TextureFormat::R8Snorm,
            Self::R8Uint => TextureFormat::R8Uint,
            Self::R8Sint => TextureFormat::R8Sint,

            // R16
            Self::R16Typeless | Self::R16Unorm => TextureFormat::R16Unorm,
            Self::R16Uint => TextureFormat::R16Uint,
            Self::R16Sint => TextureFormat::R16Sint,
            Self::R16Snorm => TextureFormat::R16Snorm,
            Self::R16Float => TextureFormat::R16Float,

            // R8G8
            Self::R8G8Typeless | Self::R8G8Unorm => TextureFormat::Rg8Unorm,
            Self::R8G8Snorm => TextureFormat::Rg8Snorm,
            Self::R8G8Uint => TextureFormat::Rg8Uint,
            Self::R8G8Sint => TextureFormat::Rg8Sint,

            // R32
            Self::R32Typeless | Self::R32Uint => TextureFormat::R32Uint,
            Self::R32Sint => TextureFormat::R32Sint,
            Self::R32Float => TextureFormat::R32Float,

            // R16G16
            Self::R16G16Typeless | Self::R16G16Unorm => TextureFormat::Rg16Unorm,
            Self::R16G16Uint => TextureFormat::Rg16Uint,
            Self::R16G16Sint => TextureFormat::Rg16Sint,
            Self::R16G16Snorm => TextureFormat::Rg16Snorm,
            Self::R16G16Float => TextureFormat::Rg16Float,

            // RGBA8
            Self::R8G8B8A8Typeless | Self::R8G8B8A8Unorm => TextureFormat::Rgba8Unorm,
            Self::R8G8B8A8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
            Self::R8G8B8A8Snorm => TextureFormat::Rgba8Snorm,
            Self::R8G8B8A8Uint => TextureFormat::Rgba8Uint,
            Self::R8G8B8A8Sint => TextureFormat::Rgba8Sint,

            // BGRA8
            Self::B8G8R8A8Typeless | Self::B8G8R8A8Unorm => TextureFormat::Bgra8Unorm,
            Self::B8G8R8A8UnormSrgb => TextureFormat::Bgra8UnormSrgb,

            // Packed formats
            Self::R9G9B9E5Sharedexp => TextureFormat::Rgb9e5Ufloat,
            Self::R10G10B10A2Typeless | Self::R10G10B10A2Unorm => TextureFormat::Rgb10a2Unorm,
            Self::R11G11B10Float => TextureFormat::Rg11b10Float,

            // R32G32
            Self::R32G32Typeless | Self::R32G32Uint => TextureFormat::Rg32Uint,
            Self::R32G32Sint => TextureFormat::Rg32Sint,
            Self::R32G32Float => TextureFormat::Rg32Float,

            // R16G16B16A16
            Self::R16G16B16A16Typeless | Self::R16G16B16A16Unorm => TextureFormat::Rgba16Unorm,
            Self::R16G16B16A16Uint => TextureFormat::Rgba16Uint,
            Self::R16G16B16A16Sint => TextureFormat::Rgba16Sint,
            Self::R16G16B16A16Snorm => TextureFormat::Rgba16Snorm,
            Self::R16G16B16A16Float => TextureFormat::Rgba16Float,

            // R32G32B32A32
            Self::R32G32B32A32Typeless | Self::R32G32B32A32Uint => TextureFormat::Rgba32Uint,
            Self::R32G32B32A32Sint => TextureFormat::Rgba32Sint,
            Self::R32G32B32A32Float => TextureFormat::Rgba32Float,

            // Depth formats.
            Self::D16Unorm => TextureFormat::Depth16Unorm,
            Self::D24UnormS8Uint => TextureFormat::Depth24PlusStencil8,
            Self::D32Float => TextureFormat::Depth32Float,

            // BC compressed formats.
            Self::Bc1Typeless | Self::Bc1Unorm => TextureFormat::Bc1RgbaUnorm,
            Self::Bc1UnormSrgb => TextureFormat::Bc1RgbaUnormSrgb,
            Self::Bc2Typeless | Self::Bc2Unorm => TextureFormat::Bc2RgbaUnorm,
            Self::Bc2UnormSrgb => TextureFormat::Bc2RgbaUnormSrgb,
            Self::Bc3Typeless | Self::Bc3Unorm => TextureFormat::Bc3RgbaUnorm,
            Self::Bc3UnormSrgb => TextureFormat::Bc3RgbaUnormSrgb,
            Self::Bc4Typeless | Self::Bc4Unorm => TextureFormat::Bc4RUnorm,
            Self::Bc4Snorm => TextureFormat::Bc4RSnorm,
            Self::Bc5Typeless | Self::Bc5Unorm => TextureFormat::Bc5RgUnorm,
            Self::Bc5Snorm => TextureFormat::Bc5RgSnorm,
            Self::Bc6HTypeless | Self::Bc6HUf16 => TextureFormat::Bc6hRgbUfloat,
            Self::Bc6HSf16 => TextureFormat::Bc6hRgbFloat,
            Self::Bc7Typeless | Self::Bc7Unorm => TextureFormat::Bc7RgbaUnorm,
            Self::Bc7UnormSrgb => TextureFormat::Bc7RgbaUnormSrgb,

            // WGPU unsupported mapping.
            _ => return Err(TextureError::UnsupportedImageFormat(*self)),
        })
    }

    /// Returns the corresponding image format from the provided DXGI_FORMAT format, or returns an error.
    pub const fn from_dxgi_format(dxgi_format: u32) -> Result<Self, TextureError> {
        if dxgi_format >= Self::Unknown as u32 && dxgi_format < Self::Count as u32 {
            // SAFETY: We check that the value is within the bounds of the enum, and that
            // the enum has no gaps or holes.
            #[allow(unsafe_code)]
            Ok(unsafe { std::mem::transmute::<u32, Self>(dxgi_format) })
        } else {
            Err(TextureError::InvalidDxgiFormat(dxgi_format))
        }
    }
}
