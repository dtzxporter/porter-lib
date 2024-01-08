use std::io::Read;
use std::io::Seek;
use std::io::Write;

use png::BitDepth;
use png::ColorType;
use png::Compression;
use png::Decoder;
use png::Encoder;
use png::SrgbRenderingIntent;
use png::Transformations;

use crate::format_to_srgb;
use crate::is_format_srgb;
use crate::Image;
use crate::ImageFileType;
use crate::ImageFormat;
use crate::TextureError;

/// Converts an image format to a png specification.
const fn format_to_png(format: ImageFormat) -> Result<(ColorType, BitDepth, bool), TextureError> {
    Ok(match format {
        ImageFormat::R1Unorm => (ColorType::Grayscale, BitDepth::One, false),
        ImageFormat::R8Unorm => (ColorType::Grayscale, BitDepth::Eight, false),
        ImageFormat::R16Unorm => (ColorType::Grayscale, BitDepth::Sixteen, false),
        ImageFormat::R8G8Unorm => (ColorType::GrayscaleAlpha, BitDepth::Eight, false),
        ImageFormat::R16G16Unorm => (ColorType::GrayscaleAlpha, BitDepth::Sixteen, false),
        ImageFormat::R8G8B8A8Unorm => (ColorType::Rgba, BitDepth::Eight, false),
        ImageFormat::R8G8B8A8UnormSrgb => (ColorType::Rgba, BitDepth::Eight, true),
        _ => {
            return Err(TextureError::ContainerFormatInvalid(
                format,
                ImageFileType::Png,
            ))
        }
    })
}

/// Creates a proper image format from the png specification.
const fn png_to_format(format: (ColorType, BitDepth)) -> Result<ImageFormat, TextureError> {
    Ok(match format {
        (ColorType::Grayscale, BitDepth::One) => ImageFormat::R1Unorm,
        (ColorType::Grayscale, BitDepth::Eight) => ImageFormat::R8Unorm,
        (ColorType::Grayscale, BitDepth::Sixteen) => ImageFormat::R16Unorm,
        (ColorType::GrayscaleAlpha, BitDepth::Eight) => ImageFormat::R8G8Unorm,
        (ColorType::GrayscaleAlpha, BitDepth::Sixteen) => ImageFormat::R16G16Unorm,
        (ColorType::Rgba, BitDepth::Eight) => ImageFormat::R8G8B8A8Unorm,
        (ColorType::Rgba, BitDepth::Sixteen) => ImageFormat::R16G16B16A16Unorm,
        _ => return Err(TextureError::UnsupportedImageFormat(ImageFormat::Unknown)),
    })
}

/// Picks the proper format required to save the input format to a png file type.
pub const fn pick_format(format: ImageFormat) -> ImageFormat {
    match format {
        // Grayscale 1bit.
        ImageFormat::R1Unorm => format,

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
        | ImageFormat::R16Uint => ImageFormat::R16Unorm,

        // Grayscale + alpha 8bit.
        ImageFormat::R8G8Typeless
        | ImageFormat::R8G8Unorm
        | ImageFormat::R8G8Uint
        | ImageFormat::R8G8Snorm
        | ImageFormat::R8G8Sint => ImageFormat::R8G8Unorm,

        // Grayscale + alpha 16bit.
        ImageFormat::R16G16Typeless
        | ImageFormat::R16G16Unorm
        | ImageFormat::R16G16Uint
        | ImageFormat::R16G16Snorm
        | ImageFormat::R16G16Sint
        | ImageFormat::R16G16Float => ImageFormat::R16G16Unorm,

        // Red + green + blue + alpha 16bit.
        ImageFormat::R16G16B16A16Typeless
        | ImageFormat::R16G16B16A16Float
        | ImageFormat::R16G16B16A16Unorm
        | ImageFormat::R16G16B16A16Uint
        | ImageFormat::R16G16B16A16Snorm
        | ImageFormat::R16G16B16A16Sint => ImageFormat::R16G16B16A16Unorm,

        // Red compressed Bc4.
        ImageFormat::Bc4Typeless | ImageFormat::Bc4Unorm | ImageFormat::Bc4Snorm => {
            ImageFormat::R8Unorm
        }

        // Various compressed formats.
        _ => {
            if is_format_srgb(format) {
                ImageFormat::R8G8B8A8UnormSrgb
            } else {
                ImageFormat::R8G8B8A8Unorm
            }
        }
    }
}

/// Writes an image to a png file to the output stream.
pub fn to_png<O: Write + Seek>(image: &Image, output: &mut O) -> Result<(), TextureError> {
    let (color_type, bit_depth, is_srgb) = format_to_png(image.format())?;

    let mut encoder = Encoder::new(output, image.width(), image.height());

    encoder.set_compression(Compression::Fast);
    encoder.set_color(color_type);
    encoder.set_depth(bit_depth);

    if is_srgb {
        encoder.set_srgb(SrgbRenderingIntent::Perceptual);
    }

    encoder.add_text_chunk("Author".into(), "DTZxPorter".into())?;

    let mut encoder = encoder.write_header()?;

    if let Some(frame) = image.frames().next() {
        let size = image.frame_size_with_mipmaps(frame.width(), frame.height(), 1);

        encoder.write_image_data(&frame.buffer()[..size as usize])?;
    }

    Ok(())
}

/// Reads a png file from the input stream to an image.
pub fn from_png<I: Read + Seek>(input: &mut I) -> Result<Image, TextureError> {
    let mut decoder = Decoder::new(input);

    decoder.set_transformations(Transformations::ALPHA);

    let mut decoder = decoder.read_info()?;

    let mut format = png_to_format(decoder.output_color_type())?;

    if decoder.info().srgb.is_some() {
        format = format_to_srgb(format);
    }

    let mut image = Image::new(decoder.info().width, decoder.info().height, format)?;
    let frame = image.create_frame(decoder.info().width, decoder.info().height)?;

    decoder.next_frame(frame.buffer_mut())?;

    Ok(image)
}
