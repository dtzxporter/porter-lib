use porter_math::Vector2;
use porter_math::Vector3;

use crate::Image;
use crate::ImageConvertOptions;
use crate::ImageFormat;
use crate::TextureError;
use crate::linear_to_srgb;
use crate::pack_unorm8;
use crate::unpack_unorm8;

/// The algorithm used to transform an image.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformAlgorithm {
    /// Reconstruct the Z channel of the image.
    /// (Requires R8G8B8A8_UNORM format)
    ReconstructZ,
    /// Reconstruct the Z channel and invert the Y channel of the image.
    /// (Requires R8G8B8A8_UNORM format)
    ReconstructZInvertY,
    /// Transform the image by scale and bias.
    UniformScaleBias(f32, f32),
}

impl TransformAlgorithm {
    /// Transforms the given image using the provided algorithm.
    pub(crate) fn transform(&self, image: &mut Image) -> Result<(), TextureError> {
        match self {
            TransformAlgorithm::ReconstructZ => reconstruct_z(image, false)?,
            TransformAlgorithm::ReconstructZInvertY => reconstruct_z(image, true)?,
            TransformAlgorithm::UniformScaleBias(scale, bias) => {
                uniform_scale_bias(image, *scale, *bias)?
            }
        }

        Ok(())
    }
}

/// Transforms the image by applying a uniform scale/bias to the pixel in linear color space.
#[inline]
fn uniform_scale_bias(image: &mut Image, scale: f32, bias: f32) -> Result<(), TextureError> {
    // Needed to track sRGB correction later if conversion isn't necessary.
    let source_format = image.format();

    if source_format.is_compressed() {
        // Handle conversion and scale/bias in a single pass.
        return image.convert(
            if source_format.is_srgb() {
                ImageFormat::R8G8B8A8UnormSrgb
            } else {
                ImageFormat::R8G8B8A8Unorm
            },
            ImageConvertOptions::UniformScaleBias(scale, bias),
        );
    } else {
        // Handle conversion to linear color when decompressing and converting.
        image.convert(ImageFormat::R8G8B8A8Unorm, ImageConvertOptions::None)?;
    }

    for frame in image.frames_mut() {
        for pixel in frame.buffer_mut().chunks_exact_mut(4) {
            let r = unpack_unorm8(pixel[0]);
            let g = unpack_unorm8(pixel[1]);
            let b = unpack_unorm8(pixel[2]);

            if source_format.is_srgb() {
                pixel[0] = pack_unorm8(linear_to_srgb((r * scale) + bias));
                pixel[1] = pack_unorm8(linear_to_srgb((g * scale) + bias));
                pixel[2] = pack_unorm8(linear_to_srgb((b * scale) + bias));
            } else {
                pixel[0] = pack_unorm8((r * scale) + bias);
                pixel[1] = pack_unorm8((g * scale) + bias);
                pixel[2] = pack_unorm8((b * scale) + bias);
            }
        }
    }

    if source_format.is_srgb() {
        image.set_format(image.format().to_srgb())?;
    }

    Ok(())
}

/// Transforms the image by reconstructing the Z (blue) channel from the XY (red, green) channels, and setting alpha to 1.0.
#[inline]
fn reconstruct_z(image: &mut Image, invert_y: bool) -> Result<(), TextureError> {
    if !matches!(image.format(), ImageFormat::R8G8B8A8Unorm) {
        return Err(TextureError::UnsupportedImageFormat(image.format()));
    }

    for frame in image.frames_mut() {
        for pixel in frame.buffer_mut().chunks_exact_mut(4) {
            let xy = Vector2::new(unpack_unorm8(pixel[0]), unpack_unorm8(pixel[1]));
            let xy_snorm = (xy * 2.0) - 1.0;

            let z = (1.0 - xy_snorm.length_squared().min(1.0)).sqrt();

            let xyz_snorm = if invert_y {
                Vector3::new(xy_snorm.x, -xy_snorm.y, z).normalized()
            } else {
                Vector3::new(xy_snorm.x, xy_snorm.y, z).normalized()
            };

            let xyz = (xyz_snorm * 0.5) + 0.5;

            pixel[0] = pack_unorm8(xyz.x);
            pixel[1] = pack_unorm8(xyz.y);
            pixel[2] = pack_unorm8(xyz.z);
            pixel[3] = 0xFF;
        }
    }

    Ok(())
}
