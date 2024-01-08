use crate::format_to_bpp;
use crate::Image;
use crate::ImageFormat;
use crate::TextureError;

/// RGBA->BGRA, BGRA->RGBA.
#[inline(always)]
fn rgba_bgra(pixel: &mut [u8]) {
    pixel.swap(0, 2);
}

/// Method used to swizzle a pixel in bytes.
type SwizzleMethod = &'static dyn Fn(&mut [u8]);

/// Matches the software swizzle method with the formats.
#[inline(always)]
fn software_swizzle_method(format: ImageFormat, target: ImageFormat) -> Option<SwizzleMethod> {
    match (format, target) {
        (ImageFormat::R8G8B8A8Unorm, ImageFormat::B8G8R8A8Unorm) => Some(&rgba_bgra),
        (ImageFormat::B8G8R8A8Unorm, ImageFormat::R8G8B8A8Unorm) => Some(&rgba_bgra),
        (ImageFormat::R8G8B8A8UnormSrgb, ImageFormat::B8G8R8A8UnormSrgb) => Some(&rgba_bgra),
        (ImageFormat::B8G8R8A8UnormSrgb, ImageFormat::R8G8B8A8UnormSrgb) => Some(&rgba_bgra),
        (ImageFormat::R8G8B8A8Typeless, ImageFormat::B8G8R8A8Typeless) => Some(&rgba_bgra),
        (ImageFormat::B8G8R8A8Typeless, ImageFormat::R8G8B8A8Typeless) => Some(&rgba_bgra),
        _ => None,
    }
}

/// Utility method for formats that can be swizzled to another format.
pub fn software_swizzle_image(image: &mut Image, format: ImageFormat) -> Result<(), TextureError> {
    let Some(swizzle) = software_swizzle_method(image.format(), format) else {
        return Err(TextureError::InvalidImageFormat(format));
    };

    let bytes_per_pixel = format_to_bpp(format) as usize / 8;

    for frame in image.frames_mut() {
        for pixel in frame.buffer_mut().chunks_exact_mut(bytes_per_pixel) {
            swizzle(pixel);
        }
    }

    Ok(())
}
