use std::io::Read;
use std::io::Seek;
use std::io::Write;

use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Image;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::TextureError;

const DDS_FOURCC: u32 = 0x00000004;
const DDS_RGB: u32 = 0x00000040;

const DDS_PIXEL_FORMAT_SRGB: u32 = 0x40000000;

const DDS_HEADER_FLAGS_TEXTURE: u32 = 0x00001007;
const DDS_HEADER_FLAGS_PITCH: u32 = 0x00000008;
const DDS_HEADER_FLAGS_LINEARSIZE: u32 = 0x00080000;
const DDS_HEADER_FLAGS_MIPMAP: u32 = 0x20000;

const DDS_SURFACE_FLAGS_TEXTURE: u32 = 0x00001000;
const DDS_SURFACE_FLAGS_CUBEMAP: u32 = 0x00000008;
const DDS_SURFACE_FLAGS_MIPMAP: u32 = 0x400008;

const DDS_CUBEMAP_ALLFACES: u32 = 0x0000FE00;

const DDS_TEX_DIMENSION_TEXTURE2D: u32 = 0x3;
const DDS_TEX_MISC_TEXTURECUBE: u32 = 0x4;

/// Utility macro used to create a FourCC code.
macro_rules! make_four_cc {
    ($x:expr, $y:expr, $z:expr, $w:expr) => {
        (($w as u32) << 24) | (($z as u32) << 16) | (($y as u32) << 8) | $x as u32
    };
}

/// Map of bitmasks to their image formats.
#[rustfmt::skip]
static BITMASK_TO_FORMAT: [(u32, u32, u32, u32, u32, ImageFormat); 9] = [
    (32, 0x000000ff, 0x0000ff00, 0x00ff0000, 0xff000000, ImageFormat::R8G8B8A8Unorm),
    (32, 0x00ff0000, 0x0000ff00, 0x000000ff, 0xff000000, ImageFormat::B8G8R8A8Unorm),
    (32, 0xffff, 0xffff0000, 0, 0, ImageFormat::R16G16Unorm),
    (24, 0xff0000, 0xff00, 0xff, 0, ImageFormat::R8G8B8Unorm),
    (16, 0x0f00, 0x00f0, 0x000f, 0xf000, ImageFormat::B4G4R4A4Unorm),
    (16, 0xff, 0, 0, 0xff00, ImageFormat::R8G8Unorm),
    (16, 0xffff, 0, 0, 0, ImageFormat::R16Unorm),
    (8, 0xff, 0, 0, 0, ImageFormat::R8Unorm),
    (8, 0, 0, 0, 0xff, ImageFormat::A8Unorm),
];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DdsPixelFormat {
    pub size: u32,
    pub flags: u32,
    pub four_cc: u32,
    pub rgb_bit_count: u32,
    pub rbit_mask: u32,
    pub gbit_mask: u32,
    pub bbit_mask: u32,
    pub abit_mask: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DdsHeader {
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    pub reserved1: [u32; 11],
    pub pixel_format: DdsPixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    pub reserved2: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DdsHeaderDx10 {
    pub dxgi_format: u32,
    pub resource_dimension: u32,
    pub misc_flag: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}

/// Calculates the pitch and slice of the given format.
fn compute_pitch_slice(format: ImageFormat, width: u32, height: u32) -> (u32, u32) {
    match format {
        ImageFormat::Bc1Typeless
        | ImageFormat::Bc1Unorm
        | ImageFormat::Bc1UnormSrgb
        | ImageFormat::Bc4Typeless
        | ImageFormat::Bc4Unorm
        | ImageFormat::Bc4Snorm => {
            let nbw = 1u32.max(width.div_ceil(4));
            let nbh = 1u32.max(height.div_ceil(4));

            let pitch = nbw * 8;
            let slice = pitch * nbh;

            (pitch, slice)
        }
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
        | ImageFormat::Bc7UnormSrgb => {
            let nbw = 1u32.max(width.div_ceil(4));
            let nbh = 1u32.max(height.div_ceil(4));

            let pitch = nbw * 16;
            let slice = pitch * nbh;

            (pitch, slice)
        }
        _ => {
            let bpp = format.bits_per_pixel();
            let pitch = (width * bpp).div_ceil(8);
            let slice = pitch * height;

            (pitch, slice)
        }
    }
}

/// Converts an image format to a pixel format and optional dx10 header.
fn format_to_pf_dx10(
    format: ImageFormat,
    array_size: u32,
    is_cubemap: bool,
) -> (DdsPixelFormat, Option<DdsHeaderDx10>) {
    let dx10_fallback = || {
        let pixel_format = DdsPixelFormat {
            size: size_of::<DdsPixelFormat>() as u32,
            flags: DDS_FOURCC,
            four_cc: make_four_cc!('D', 'X', '1', '0'),
            rgb_bit_count: 0,
            rbit_mask: 0,
            gbit_mask: 0,
            bbit_mask: 0,
            abit_mask: 0,
        };

        let header_dx10 = DdsHeaderDx10 {
            dxgi_format: format as u32,
            resource_dimension: DDS_TEX_DIMENSION_TEXTURE2D,
            misc_flag: if is_cubemap {
                DDS_TEX_MISC_TEXTURECUBE
            } else {
                0
            },
            array_size,
            misc_flags2: 0,
        };

        (pixel_format, Some(header_dx10))
    };

    if array_size > 1 && !is_cubemap {
        return dx10_fallback();
    }

    match format {
        ImageFormat::Bc1Unorm => (
            DdsPixelFormat {
                size: size_of::<DdsPixelFormat>() as u32,
                flags: DDS_FOURCC,
                four_cc: make_four_cc!('D', 'X', 'T', '1'),
                rgb_bit_count: 0,
                rbit_mask: 0,
                gbit_mask: 0,
                bbit_mask: 0,
                abit_mask: 0,
            },
            None,
        ),
        ImageFormat::Bc2Unorm => (
            DdsPixelFormat {
                size: size_of::<DdsPixelFormat>() as u32,
                flags: DDS_FOURCC,
                four_cc: make_four_cc!('D', 'X', 'T', '3'),
                rgb_bit_count: 0,
                rbit_mask: 0,
                gbit_mask: 0,
                bbit_mask: 0,
                abit_mask: 0,
            },
            None,
        ),
        ImageFormat::Bc3Unorm => (
            DdsPixelFormat {
                size: size_of::<DdsPixelFormat>() as u32,
                flags: DDS_FOURCC,
                four_cc: make_four_cc!('D', 'X', 'T', '5'),
                rgb_bit_count: 0,
                rbit_mask: 0,
                gbit_mask: 0,
                bbit_mask: 0,
                abit_mask: 0,
            },
            None,
        ),
        ImageFormat::Bc4Unorm => (
            DdsPixelFormat {
                size: size_of::<DdsPixelFormat>() as u32,
                flags: DDS_FOURCC,
                four_cc: make_four_cc!('A', 'T', 'I', '1'),
                rgb_bit_count: 0,
                rbit_mask: 0,
                gbit_mask: 0,
                bbit_mask: 0,
                abit_mask: 0,
            },
            None,
        ),
        ImageFormat::Bc5Unorm => (
            DdsPixelFormat {
                size: size_of::<DdsPixelFormat>() as u32,
                flags: DDS_FOURCC,
                four_cc: make_four_cc!('A', 'T', 'I', '2'),
                rgb_bit_count: 0,
                rbit_mask: 0,
                gbit_mask: 0,
                bbit_mask: 0,
                abit_mask: 0,
            },
            None,
        ),
        ImageFormat::R8G8B8Unorm => (
            DdsPixelFormat {
                size: size_of::<DdsPixelFormat>() as u32,
                flags: DDS_RGB,
                four_cc: 0,
                rgb_bit_count: 24,
                rbit_mask: 0xff0000,
                gbit_mask: 0xff00,
                bbit_mask: 0xff,
                abit_mask: 0x0,
            },
            None,
        ),
        // Fall back to directx 10 header, which uses format directly.
        _ => dx10_fallback(),
    }
}

/// Creates a header, and optional dx10 header for an image.
fn format_to_dds(image: &Image) -> (DdsHeader, Option<DdsHeaderDx10>) {
    let mut caps: u32 = DDS_SURFACE_FLAGS_TEXTURE;
    let mut flags: u32 = DDS_HEADER_FLAGS_TEXTURE;

    let is_cubemap = image.is_cubemap();

    let caps2 = if is_cubemap {
        caps |= DDS_SURFACE_FLAGS_CUBEMAP;
        DDS_CUBEMAP_ALLFACES
    } else {
        0
    };

    let mip_map_count = image.mipmaps();

    if mip_map_count > 0 {
        caps |= DDS_SURFACE_FLAGS_MIPMAP;
        flags |= DDS_HEADER_FLAGS_MIPMAP;
    }

    let (pitch, slice) = compute_pitch_slice(image.format(), image.width(), image.height());

    let pitch_or_linear_size = if image.format().is_compressed() {
        flags |= DDS_HEADER_FLAGS_LINEARSIZE;
        slice
    } else {
        flags |= DDS_HEADER_FLAGS_PITCH;
        pitch
    };

    let (pixel_format, header_dx10) =
        format_to_pf_dx10(image.format(), image.frames().len() as u32, is_cubemap);

    let header = DdsHeader {
        size: size_of::<DdsHeader>() as u32,
        flags,
        height: image.height(),
        width: image.width(),
        pitch_or_linear_size,
        depth: 1,
        mip_map_count,
        reserved1: [0; 11],
        pixel_format,
        caps,
        caps2,
        caps3: 0,
        caps4: 0,
        reserved2: 0,
    };

    (header, header_dx10)
}

/// Creates a proper image format from the dds pixel format.
fn dds_to_format(pixel_format: &DdsPixelFormat) -> Result<ImageFormat, TextureError> {
    if (pixel_format.flags & DDS_FOURCC) > 0 {
        let format = match pixel_format.four_cc {
            // Standard block encoded values.
            x if x == make_four_cc!('D', 'X', 'T', '1') => ImageFormat::Bc1Unorm,
            x if x == make_four_cc!('D', 'X', 'T', '3') => ImageFormat::Bc2Unorm,
            x if x == make_four_cc!('D', 'X', 'T', '5') => ImageFormat::Bc3Unorm,
            x if x == make_four_cc!('A', 'T', 'I', '1') => ImageFormat::Bc4Unorm,
            x if x == make_four_cc!('A', 'T', 'I', '2') => ImageFormat::Bc5Unorm,
            x if x == make_four_cc!('A', '2', 'X', 'Y') => ImageFormat::Bc5Unorm,
            x if x == make_four_cc!('B', 'C', '4', 'U') => ImageFormat::Bc4Unorm,
            x if x == make_four_cc!('B', 'C', '4', 'S') => ImageFormat::Bc4Snorm,
            x if x == make_four_cc!('B', 'C', '5', 'U') => ImageFormat::Bc5Unorm,
            x if x == make_four_cc!('B', 'C', '5', 'S') => ImageFormat::Bc5Snorm,

            // D3DFormat legacy values.
            36 => ImageFormat::R16G16B16A16Unorm,
            110 => ImageFormat::R16G16B16A16Unorm,
            111 => ImageFormat::R16Float,
            112 => ImageFormat::R16G16Float,
            113 => ImageFormat::R16G16B16A16Float,
            114 => ImageFormat::R32Float,
            115 => ImageFormat::R32G32Float,
            116 => ImageFormat::R32G32B32A32Float,

            // Unknown or unsupported.
            _ => {
                #[cfg(debug_assertions)]
                println!("Unsupported fourCC code: {:#02X?}", pixel_format.four_cc);

                return Err(TextureError::UnsupportedImageFormat(ImageFormat::Unknown));
            }
        };

        return Ok(format);
    }

    for mask in &BITMASK_TO_FORMAT {
        if pixel_format.rgb_bit_count == mask.0
            && pixel_format.rbit_mask == mask.1
            && pixel_format.gbit_mask == mask.2
            && pixel_format.bbit_mask == mask.3
            && pixel_format.abit_mask == mask.4
        {
            return Ok(mask.5);
        }
    }

    #[cfg(debug_assertions)]
    println!("Unsupported masks: {:#02X?}", pixel_format);

    Err(TextureError::UnsupportedImageFormat(ImageFormat::Unknown))
}

/// Picks the proper format required to save the input format to a dds file type.
pub const fn pick_format(format: ImageFormat) -> ImageFormat {
    match format {
        ImageFormat::B8G8R8Unorm => ImageFormat::R8G8B8A8Unorm,
        ImageFormat::A8R8G8B8Unorm => ImageFormat::R8G8B8A8Unorm,
        _ => format,
    }
}

/// Writes an image to a dds file to the output stream.
pub fn to_dds<O: Write + Seek>(image: &Image, output: &mut O) -> Result<(), TextureError> {
    let (header, header_dx10) = format_to_dds(image);

    output.write_all(&make_four_cc!('D', 'D', 'S', ' ').to_le_bytes())?;

    output.write_struct(header)?;

    if let Some(header_dx10) = header_dx10 {
        output.write_struct(header_dx10)?;
    }

    for frame in image.frames() {
        output.write_all(frame.buffer())?;
    }

    Ok(())
}

/// Reads a dds file from the input stream to an image.
pub fn from_dds<I: Read + Seek>(input: &mut I) -> Result<Image, TextureError> {
    let magic: u32 = input.read_struct()?;

    if magic != make_four_cc!('D', 'D', 'S', ' ') {
        return Err(TextureError::ContainerInvalid(ImageFileType::Dds));
    }

    let header: DdsHeader = input.read_struct()?;

    let mut frames = if header.caps2 & DDS_CUBEMAP_ALLFACES == DDS_CUBEMAP_ALLFACES {
        6
    } else {
        1
    };

    let mut format: ImageFormat =
        if header.pixel_format.four_cc == make_four_cc!('D', 'X', '1', '0') {
            let header_dx10: DdsHeaderDx10 = input.read_struct()?;

            frames = frames.max(header_dx10.array_size);

            ImageFormat::from_dxgi_format(header_dx10.dxgi_format)?
        } else {
            dds_to_format(&header.pixel_format)?
        };

    if header.reserved1[9] == make_four_cc!('N', 'V', 'T', 'T')
        && (header.pixel_format.flags & DDS_PIXEL_FORMAT_SRGB) > 0
    {
        format = format.to_srgb();
    }

    let mut image = Image::with_mipmaps(
        header.width,
        header.height,
        header.mip_map_count.max(1),
        format,
    )?;

    for _ in 0..frames {
        let frame = image.create_frame()?;

        input.read_exact(frame.buffer_mut())?;
    }

    Ok(image)
}
