use crate::Frame;
use crate::Image;
use crate::TextureError;

/// The algorithm used to resize an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAlgorithm {
    // Each pixel in the new image samples 4x4 pixels around the closest pixel in the original image.
    Bicubic,
    // Each pixel in the new image takes the value of its nearest pixel in the original image.
    NearestNeighbor,
}

impl ResizeAlgorithm {
    /// Resizes the given image using the new dimensions and source image.
    pub(crate) fn resize(
        &self,
        image: &mut Image,
        width: u32,
        height: u32,
    ) -> Result<(), TextureError> {
        let mut result = Image::new(width, height, image.format())?;

        for src in image.frames() {
            let dest = result.create_frame()?;

            let scale_x = image.width() as f32 / width as f32;
            let scale_y = image.height() as f32 / height as f32;

            for (index, pixel) in dest
                .buffer_mut()
                .chunks_exact_mut(4)
                .take(width as usize * height as usize)
                .enumerate()
            {
                let x = index % width as usize;
                let y = index / width as usize;

                let src_x = x as f32 * scale_x;
                let src_y = y as f32 * scale_y;

                let new_pixel = match self {
                    ResizeAlgorithm::Bicubic => {
                        interpolate_bicubic(src, image.width(), image.height(), src_x, src_y)
                    }
                    ResizeAlgorithm::NearestNeighbor => interpolate_nearest_neighbor(
                        src,
                        image.width(),
                        image.height(),
                        src_x,
                        src_y,
                    ),
                };

                pixel[0] = new_pixel[0];
                pixel[1] = new_pixel[1];
                pixel[2] = new_pixel[2];
                pixel[3] = new_pixel[3];
            }
        }

        *image = result;

        Ok(())
    }
}

/// Interpolates a pixel using a nearest neighbor filter.
#[inline]
fn interpolate_nearest_neighbor(src: &Frame, width: u32, height: u32, x: f32, y: f32) -> [u8; 4] {
    let x0 = x.round().min(width as f32 - 1.0) as usize;
    let y0 = y.round().min(height as f32 - 1.0) as usize;

    if x0 < width as usize && y0 < height as usize {
        let src_index = (y0 * width as usize + x0) * 4;
        let src_buffer = src.buffer();

        [
            src_buffer[src_index],
            src_buffer[src_index + 1],
            src_buffer[src_index + 2],
            src_buffer[src_index + 3],
        ]
    } else {
        [0, 0, 0, 0]
    }
}

/// Interpolates a pixel using a bicubic filter.
#[inline]
fn interpolate_bicubic(src: &Frame, width: u32, height: u32, x: f32, y: f32) -> [u8; 4] {
    let mut result: [f32; 4] = [0.0; 4];
    let mut total_weight = 0.0;

    let x0 = x.floor() as isize;
    let y0 = y.floor() as isize;

    let cubic_spline = |x: f32, b: f32, c: f32| -> f32 {
        let a = x.abs();

        let k = if a < 1.0 {
            (12.0 - 9.0 * b - 6.0 * c) * a.powi(3)
                + (-18.0 + 12.0 * b + 6.0 * c) * a.powi(2)
                + (6.0 - 2.0 * b)
        } else if a < 2.0 {
            (-b - 6.0 * c) * a.powi(3)
                + (6.0 * b + 30.0 * c) * a.powi(2)
                + (-12.0 * b - 48.0 * c) * a
                + (8.0 * b + 24.0 * c)
        } else {
            0.0
        };

        k / 6.0
    };

    let cubic = |t: f32| cubic_spline(t, 0.0, 0.5);

    for dy in -1..=2 {
        for dx in -1..=2 {
            let nx = x0 + dx;
            let ny = y0 + dy;

            if nx >= 0 && nx < width as isize && ny >= 0 && ny < height as isize {
                let weight = cubic(x - nx as f32) * cubic(y - ny as f32);
                let src_index = (ny as usize * width as usize + nx as usize) * 4;
                let src_buffer = src.buffer();

                total_weight += weight;

                result[0] += weight * src_buffer[src_index] as f32;
                result[1] += weight * src_buffer[src_index + 1] as f32;
                result[2] += weight * src_buffer[src_index + 2] as f32;
                result[3] += weight * src_buffer[src_index + 3] as f32;
            }
        }
    }

    if total_weight > 0.0 {
        for pixel in &mut result {
            *pixel /= total_weight;
            *pixel = pixel.clamp(0.0, 255.0);
        }
    }

    [
        result[0] as u8,
        result[1] as u8,
        result[2] as u8,
        result[3] as u8,
    ]
}
