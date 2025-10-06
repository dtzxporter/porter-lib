use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use porter_utils::SeekExt;
use porter_utils::StackVec;
use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Image;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::TextureError;

/// Maximum number of tga frames to expand.
const MAXIMUM_TGA_FRAMES: usize = 6;
/// Maximum run-length chunk size.
const MAXIMUM_RLE_LENGTH: usize = 128;
/// The maximum bytes per pixel for a tga.
const MAXIMUM_BYTES_PER_PIXEL: usize = 4;
/// The maximum run-length buffer size.
const MAXIMUM_RLE_BUFFER: usize = MAXIMUM_BYTES_PER_PIXEL * MAXIMUM_RLE_LENGTH;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum ImageType {
    UncompressedRgb = 2,
    UncompressedGrayscale = 3,
    CompressedRgb = 10,
    CompressedGrayscale = 11,
}

#[derive(Debug, Clone, Copy)]
enum ColorType {
    Gray,
    Rgba,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct TgaHeader {
    id_size: u8,
    color_type: u8,
    image_type: u8,
    color_map_origin: u16,
    color_map_length: u16,
    color_map_depth: u8,
    x_origin: u16,
    y_origin: u16,
    width: u16,
    height: u16,
    bits_per_pixel: u8,
    image_descriptor: u8,
}

/// Converts an image format to a tga specification.
const fn format_to_tga(format: ImageFormat) -> Result<(ColorType, ImageType, u8), TextureError> {
    Ok(match format {
        ImageFormat::R8Unorm => (ColorType::Gray, ImageType::CompressedGrayscale, 8),
        ImageFormat::B8G8R8A8Unorm => (ColorType::Rgba, ImageType::CompressedRgb, 32),
        ImageFormat::B8G8R8A8UnormSrgb => (ColorType::Rgba, ImageType::CompressedRgb, 32),
        _ => {
            return Err(TextureError::ContainerFormatInvalid(
                format,
                ImageFileType::Tga,
            ));
        }
    })
}

/// Picks the proper format required to save the input format to a tga file type.
pub const fn pick_format(format: ImageFormat) -> ImageFormat {
    match format {
        // Grayscale 1bit.
        ImageFormat::R1Unorm => ImageFormat::R8Unorm,

        // Grayscale 8bit.
        ImageFormat::R8Typeless
        | ImageFormat::R8Unorm
        | ImageFormat::R8Sint
        | ImageFormat::R8Uint => ImageFormat::R8Unorm,

        // Grayscale 16bit.
        ImageFormat::R16Typeless
        | ImageFormat::R16Float
        | ImageFormat::R16Unorm
        | ImageFormat::R16Snorm
        | ImageFormat::R16Sint
        | ImageFormat::R16Uint => ImageFormat::R8Unorm,

        // Red compressed Bc4.
        ImageFormat::Bc4Typeless | ImageFormat::Bc4Unorm | ImageFormat::Bc4Snorm => {
            ImageFormat::R8Unorm
        }

        // Various compressed formats.
        _ => {
            if format.is_srgb() {
                ImageFormat::B8G8R8A8UnormSrgb
            } else {
                ImageFormat::B8G8R8A8Unorm
            }
        }
    }
}

/// Writes an image to a tga file to the output stream.
pub fn to_tga<O: Write + Seek>(image: &Image, output: &mut O) -> Result<(), TextureError> {
    let (color_type, image_type, bit_depth) = format_to_tga(image.format())?;

    let frames = image.frames();
    let width = image.width();
    let height = image.height() * frames.len().min(MAXIMUM_TGA_FRAMES) as u32;

    if width > u16::MAX as u32 || height > u16::MAX as u32 {
        return Err(TextureError::InvalidImageSize(width, height));
    }

    let header = TgaHeader {
        id_size: 0,
        color_type: 0,
        image_type: image_type as u8,
        color_map_origin: 0,
        color_map_length: 0,
        color_map_depth: 0,
        x_origin: 0,
        y_origin: 0,
        width: width as u16,
        height: height as u16,
        bits_per_pixel: bit_depth,
        image_descriptor: 32,
    };

    output.write_struct(header)?;

    let size = image.frame_size_with_mipmaps(image.width(), image.height(), 1);
    let stride = match color_type {
        ColorType::Gray => image.width() as usize,
        ColorType::Rgba => image.width() as usize * 4,
    };

    for frame in frames.iter().take(MAXIMUM_TGA_FRAMES) {
        match color_type {
            ColorType::Gray => {
                write_rle_encode::<1, _>(&frame.buffer()[..size as usize], stride, output)?
            }
            ColorType::Rgba => {
                write_rle_encode::<4, _>(&frame.buffer()[..size as usize], stride, output)?
            }
        };
    }

    Ok(())
}

/// Reads a tga file from the input stream to an image.
pub fn from_tga<I: Read + Seek>(input: &mut I) -> Result<Image, TextureError> {
    let header: TgaHeader = input.read_struct()?;

    input.skip(header.id_size)?;

    if header.color_type != 0 {
        return Err(TextureError::ContainerInvalid(ImageFileType::Tga));
    }

    if header.x_origin != 0 || header.y_origin != 0 {
        return Err(TextureError::ContainerInvalid(ImageFileType::Tga));
    }

    let format = match header.bits_per_pixel {
        8 => ImageFormat::R8Unorm,
        32 => ImageFormat::B8G8R8A8Unorm,
        _ => return Err(TextureError::ContainerInvalid(ImageFileType::Tga)),
    };

    let mut image = Image::new(header.width as u32, header.height as u32, format)?;
    let frame = image.create_frame()?;

    match header.image_type {
        x if x == ImageType::UncompressedRgb as u8 => {
            input.read_exact(frame.buffer_mut())?;
        }
        x if x == ImageType::UncompressedGrayscale as u8 => {
            input.read_exact(frame.buffer_mut())?;
        }
        x if x == ImageType::CompressedRgb as u8 => {
            read_rle_decode::<4, _>(frame.buffer_mut(), input)?;
        }
        x if x == ImageType::CompressedGrayscale as u8 => {
            read_rle_decode::<1, _>(frame.buffer_mut(), input)?;
        }
        _ => return Err(TextureError::ContainerInvalid(ImageFileType::Tga)),
    }

    Ok(image)
}

/// Utility method to read a run-length frame and decode it.
fn read_rle_decode<const BYTES_PER_PIXEL: usize, I: Read + Seek>(
    buffer: &mut [u8],
    input: &mut I,
) -> Result<(), TextureError> {
    let length = buffer.len() as u64;

    let mut writer = Cursor::new(buffer);

    while writer.position() < length {
        let opcode: u8 = input.read_struct()?;

        if (opcode & 0x80) != 0 {
            let len = ((opcode & !0x80) + 1) as usize;
            let pixel: [u8; BYTES_PER_PIXEL] = input.read_struct()?;

            for _ in 0..len {
                writer.write_all(&pixel)?;
            }
        } else {
            let len = (opcode + 1) as u64 * BYTES_PER_PIXEL as u64;

            std::io::copy(&mut input.take(len), &mut writer)?;
        }
    }

    Ok(())
}

/// Utility method to write a frame run-length encoded.
fn write_rle_encode<const BYTES_PER_PIXEL: usize, O: Write + Seek>(
    buffer: &[u8],
    stride: usize,
    output: &mut O,
) -> Result<(), TextureError> {
    let mut scratch = StackVec::new([0; MAXIMUM_RLE_BUFFER]);

    for row in buffer.chunks_exact(stride) {
        let mut counter = 0;
        let mut prev_pixel: Option<&[u8]> = None;
        let mut packet_type_rle = true;

        for pixel in row.chunks_exact(BYTES_PER_PIXEL) {
            if let Some(prev_pixel) = prev_pixel {
                if pixel == prev_pixel {
                    if !packet_type_rle && counter > 2 {
                        write_raw(
                            &scratch[0..scratch.len() - BYTES_PER_PIXEL],
                            counter as u8 - 1,
                            output,
                        )?;

                        counter = 1;
                        scratch.clear();
                    }

                    packet_type_rle = true;
                } else if packet_type_rle && counter > 0 {
                    write_rle(prev_pixel, counter as u8, output)?;

                    counter = 0;
                    packet_type_rle = false;
                    scratch.clear();
                }
            }

            counter += 1;
            scratch.write_all(pixel)?;

            if counter == MAXIMUM_RLE_LENGTH {
                if packet_type_rle {
                    write_rle(prev_pixel.unwrap_or_default(), counter as u8, output)?;
                } else {
                    write_raw(&scratch, counter as u8, output)?;
                }

                counter = 0;
                packet_type_rle = true;
                scratch.clear();
            }

            prev_pixel = Some(pixel);
        }

        if counter > 0 {
            if packet_type_rle {
                write_rle(prev_pixel.unwrap_or_default(), counter as u8, output)?;
            } else {
                write_raw(&scratch, counter as u8, output)?;
            }
        }

        scratch.clear();
    }

    Ok(())
}

/// Utility method to write a raw opcode.
#[inline]
fn write_raw<O: Write + Seek>(
    buffer: &[u8],
    counter: u8,
    output: &mut O,
) -> Result<(), TextureError> {
    output.write_struct(counter - 1)?;
    output.write_all(buffer)?;

    Ok(())
}

/// Utility method to write a run-length opcode.
#[inline]
fn write_rle<O: Write + Seek>(
    buffer: &[u8],
    counter: u8,
    output: &mut O,
) -> Result<(), TextureError> {
    output.write_struct(0x80 | (counter - 1))?;
    output.write_all(buffer)?;

    Ok(())
}
