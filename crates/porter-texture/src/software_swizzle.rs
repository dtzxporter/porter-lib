use crate::Image;
use crate::ImageFormat;
use crate::TextureError;

/// RGBA->BGRA, BGRA->RGBA.
#[inline(always)]
fn rgba_bgra(pixel: &mut [u8]) {
    pixel.swap(0, 2);
}

/// RGBA->ARGB.
#[inline(always)]
fn rgba_argb(pixel: &mut [u8]) {
    pixel.swap(0, 1); // GRBA
    pixel.swap(0, 2); // BRGA
    pixel.swap(0, 3); // ARGB
}

/// ARGB->RGBA.
#[inline(always)]
fn argb_rgba(pixel: &mut [u8]) {
    pixel.swap(0, 3); // BRGA
    pixel.swap(0, 2); // GRBA
    pixel.swap(0, 1); // RGBA
}

/// BGRA->ARGB, ARGB->BGRA.
#[inline(always)]
fn bgra_argb(pixel: &mut [u8]) {
    pixel.swap(0, 3); // AGRB
    pixel.swap(1, 2); // ARGB
}

/// Method used to swizzle a pixel in bytes.
type SwizzleMethod = fn(&mut [u8]);

/// Matches the software swizzle method with the formats.
#[inline(always)]
fn software_swizzle_method(format: ImageFormat, target: ImageFormat) -> Option<SwizzleMethod> {
    match (format, target) {
        (ImageFormat::R8G8B8Unorm, ImageFormat::B8G8R8Unorm) => Some(rgba_bgra),
        (ImageFormat::B8G8R8Unorm, ImageFormat::R8G8B8Unorm) => Some(rgba_bgra),
        (ImageFormat::R8G8B8A8Unorm, ImageFormat::B8G8R8A8Unorm) => Some(rgba_bgra),
        (ImageFormat::R8G8B8A8Unorm, ImageFormat::A8R8G8B8Unorm) => Some(rgba_argb),
        (ImageFormat::B8G8R8A8Unorm, ImageFormat::R8G8B8A8Unorm) => Some(rgba_bgra),
        (ImageFormat::B8G8R8A8Unorm, ImageFormat::A8R8G8B8Unorm) => Some(bgra_argb),
        (ImageFormat::A8R8G8B8Unorm, ImageFormat::R8G8B8A8Unorm) => Some(argb_rgba),
        (ImageFormat::A8R8G8B8Unorm, ImageFormat::B8G8R8A8Unorm) => Some(bgra_argb),
        (ImageFormat::R8G8B8A8UnormSrgb, ImageFormat::B8G8R8A8UnormSrgb) => Some(rgba_bgra),
        (ImageFormat::B8G8R8A8UnormSrgb, ImageFormat::R8G8B8A8UnormSrgb) => Some(rgba_bgra),
        (ImageFormat::R8G8B8A8Typeless, ImageFormat::B8G8R8A8Typeless) => Some(rgba_bgra),
        (ImageFormat::B8G8R8A8Typeless, ImageFormat::R8G8B8A8Typeless) => Some(rgba_bgra),
        _ => None,
    }
}

/// Utility method for formats that can be swizzled to another format.
pub fn software_swizzle_image(image: &mut Image, format: ImageFormat) -> Result<(), TextureError> {
    let Some(swizzle) = software_swizzle_method(image.format(), format) else {
        return Err(TextureError::InvalidImageFormat(format));
    };

    let bytes_per_pixel = format.bits_per_pixel() as usize / 8;

    for frame in image.frames_mut() {
        for pixel in frame.buffer_mut().chunks_exact_mut(bytes_per_pixel) {
            swizzle(pixel);
        }
    }

    Ok(())
}
