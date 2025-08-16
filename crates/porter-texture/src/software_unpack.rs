use std::io::Cursor;
use std::io::Write;

use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Image;
use crate::ImageFormat;
use crate::TextureError;

/// Utility method for interlacing an extra pixel component into an existing pixel.
fn unpack_extra_component(
    image: &mut Image,
    format: ImageFormat,
    stride: usize,
    component: &[u8],
) -> Result<(), TextureError> {
    let mut result = Image::with_mipmaps(image.width(), image.height(), image.mipmaps(), format)?;

    for frame in image.frames() {
        let new_frame = result.create_frame()?;

        let mut writer = Cursor::new(new_frame.buffer_mut());

        for pixel in frame.buffer().chunks_exact(stride) {
            writer.write_all(pixel)?;
            writer.write_all(component)?;
        }
    }

    *image = result;

    Ok(())
}

/// Utility method for formats that require unpacking before conversion.
pub fn software_unpack_image(image: &mut Image) -> Result<(), TextureError> {
    match image.format() {
        ImageFormat::R1Unorm => {
            let mut result = Image::new(image.width(), image.height(), ImageFormat::R8Unorm)?;

            for frame in image.frames() {
                let new_frame = result.create_frame()?;

                let mut reader = Cursor::new(frame.buffer());
                let mut writer = Cursor::new(new_frame.buffer_mut());

                for _ in 0..image.height() {
                    let nbw = image.width().div_ceil(8);
                    let mut unpacked = 0;

                    for _ in 0..nbw {
                        let pixel: u8 = reader.read_struct()?;

                        for bcount in 0..8 {
                            if unpacked >= image.width() {
                                break;
                            }

                            if ((pixel >> (7 - bcount)) & 0x1) > 0 {
                                writer.write_struct(0xFFu8)?;
                            } else {
                                writer.write_struct(0x0u8)?;
                            }

                            unpacked += 1;
                        }
                    }
                }
            }

            *image = result;
        }
        ImageFormat::R8G8B8Unorm => {
            unpack_extra_component(image, ImageFormat::R8G8B8A8Unorm, 3, &[0xFF])?;
        }
        ImageFormat::B8G8R8Unorm => {
            unpack_extra_component(image, ImageFormat::B8G8R8A8Unorm, 3, &[0xFF])?;
        }
        ImageFormat::R32G32B32Typeless => {
            unpack_extra_component(
                image,
                ImageFormat::R32G32B32A32Typeless,
                12,
                &[0xFF, 0xFF, 0xFF, 0xFF],
            )?;
        }
        ImageFormat::R32G32B32Float => {
            unpack_extra_component(
                image,
                ImageFormat::R32G32B32A32Float,
                12,
                &[0x00, 0x00, 0x80, 0x3F],
            )?;
        }
        ImageFormat::R32G32B32Uint => {
            unpack_extra_component(
                image,
                ImageFormat::R32G32B32A32Uint,
                12,
                &[0xFF, 0xFF, 0xFF, 0xFF],
            )?;
        }
        ImageFormat::R32G32B32Sint => {
            unpack_extra_component(
                image,
                ImageFormat::R32G32B32A32Sint,
                12,
                &[0x80, 0x80, 0x80, 0x80],
            )?;
        }
        _ => return Err(TextureError::ConversionError),
    }

    Ok(())
}
