use std::io::Seek;
use std::io::Write;

use porter_utils::AsByteSlice;

use crate::format_to_bpp;
use crate::is_format_compressed;
use crate::Image;
use crate::ImageFormat;
use crate::TextureError;

const DDS_FOURCC: u32 = 0x00000004;

const DDS_HEADER_FLAGS_TEXTURE: u32 = 0x00001007;
const DDS_HEADER_FLAGS_PITCH: u32 = 0x00000008;
const DDS_HEADER_FLAGS_LINEARSIZE: u32 = 0x00080000;

const DDS_SURFACE_FLAGS_TEXTURE: u32 = 0x00001000;
const DDS_SURFACE_FLAGS_CUBEMAP: u32 = 0x00000008;

const DDS_CUBEMAP_ALLFACES: u32 = 0x0000FE00;

const DDS_TEX_DIMENSION_TEXTURE2D: u32 = 0x3;
const DDS_TEX_MISC_TEXTURECUBE: u32 = 0x4;

/// Utility macro used to create a FourCC code.
macro_rules! make_four_cc {
    ($x:expr, $y:expr, $z:expr, $w:expr) => {
        (($w as u32) << 24) | (($z as u32) << 16) | (($y as u32) << 8) | $x as u32
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
struct DdsHeaderDx10 {
    pub dxgi_format: u32,
    pub resource_dimension: u32,
    pub misc_flag: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}

/// Calculates the pitch and slice of  the given format.
fn compute_pitch_slice(format: ImageFormat, width: u32, height: u32) -> (u32, u32) {
    match format {
        ImageFormat::Bc1Typeless
        | ImageFormat::Bc1Unorm
        | ImageFormat::Bc1UnormSrgb
        | ImageFormat::Bc4Typeless
        | ImageFormat::Bc4Unorm
        | ImageFormat::Bc4Snorm => {
            let nbw = 1u32.max((width + 3) / 4);
            let nbh = 1u32.max((height + 3) / 4);

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
            let nbw = 1u32.max((width + 3) / 4);
            let nbh = 1u32.max((height + 3) / 4);

            let pitch = nbw * 16;
            let slice = pitch * nbh;

            (pitch, slice)
        }
        _ => {
            let bpp = format_to_bpp(format);
            let pitch = (width * bpp + 7) / 8;
            let slice = pitch * height;

            (pitch, slice)
        }
    }
}

/// Converts an image format to a pixel format and optional dx10 header.
const fn format_to_pf_dx10(
    format: ImageFormat,
    is_cubemap: bool,
) -> (DdsPixelFormat, Option<DdsHeaderDx10>) {
    match format {
        ImageFormat::Bc1Unorm => (
            DdsPixelFormat {
                size: std::mem::size_of::<DdsPixelFormat>() as u32,
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
                size: std::mem::size_of::<DdsPixelFormat>() as u32,
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
                size: std::mem::size_of::<DdsPixelFormat>() as u32,
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
                size: std::mem::size_of::<DdsPixelFormat>() as u32,
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
                size: std::mem::size_of::<DdsPixelFormat>() as u32,
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
        // Fall back to directx 10 header, which uses format directly.
        _ => {
            let pixel_format = DdsPixelFormat {
                size: std::mem::size_of::<DdsPixelFormat>() as u32,
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
                array_size: if is_cubemap { 6 } else { 1 },
                misc_flags2: 0,
            };

            (pixel_format, Some(header_dx10))
        }
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

    let (pitch, slice) = compute_pitch_slice(image.format(), image.width(), image.height());

    let pitch_or_linear_size = if is_format_compressed(image.format()) {
        flags |= DDS_HEADER_FLAGS_LINEARSIZE;
        slice
    } else {
        flags |= DDS_HEADER_FLAGS_PITCH;
        pitch
    };

    let (pixel_format, header_dx10) = format_to_pf_dx10(image.format(), is_cubemap);

    let header = DdsHeader {
        size: std::mem::size_of::<DdsHeader>() as u32,
        flags,
        height: image.height(),
        width: image.width(),
        pitch_or_linear_size,
        depth: 1,
        mip_map_count: 0,
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

/// Picks the proper format required to save the input format to a dds file type.
pub const fn pick_format(format: ImageFormat) -> ImageFormat {
    format
}

/// Writes an image to a dds file to the output stream.
pub fn to_dds<O: Write + Seek>(image: &Image, output: &mut O) -> Result<(), TextureError> {
    let (header, header_dx10) = format_to_dds(image);

    output.write_all(&make_four_cc!('D', 'D', 'S', ' ').to_le_bytes())?;

    output.write_all(header.as_byte_slice())?;

    if let Some(header_dx10) = header_dx10 {
        output.write_all(header_dx10.as_byte_slice())?;
    }

    for frame in image.frames() {
        output.write_all(frame.buffer())?;
    }

    Ok(())
}
