use std::io::Cursor;

use porter_utils::StructReadExt;
use porter_utils::StructWriteExt;

use crate::Image;
use crate::ImageFormat;
use crate::TextureError;

/// Utility method for formats that require unpacking before conversion.
pub fn software_unpack_image(image: &mut Image) -> Result<(), TextureError> {
    match image.format() {
        ImageFormat::R1Unorm => {
            let mut result = Image::new(image.width(), image.height(), ImageFormat::R8Unorm)?;

            for frame in image.frames() {
                let new_frame = result.create_frame(frame.width(), frame.height())?;

                let mut reader = Cursor::new(frame.buffer());
                let mut writer = Cursor::new(new_frame.buffer_mut());

                for _ in 0..frame.height() {
                    let nbw = (frame.width() + 7) / 8;
                    let mut unpacked = 0;

                    for _ in 0..nbw {
                        let pixel: u8 = reader.read_struct()?;

                        for bcount in 0..8 {
                            if unpacked >= frame.width() {
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
            let mut result = Image::new(image.width(), image.height(), ImageFormat::R8G8B8A8Unorm)?;

            for frame in image.frames() {
                let new_frame = result.create_frame(frame.width(), frame.height())?;

                let mut reader = Cursor::new(frame.buffer());
                let mut writer = Cursor::new(new_frame.buffer_mut());

                for _ in 0..frame.height() {
                    for _ in 0..frame.width() {
                        let pixel: [u8; 3] = reader.read_struct()?;
                        let pixel: [u8; 4] = [pixel[0], pixel[1], pixel[2], 0xFF];

                        writer.write_struct(pixel)?;
                    }
                }
            }

            *image = result;
        }
        _ => return Err(TextureError::ConversionError),
    }

    Ok(())
}
