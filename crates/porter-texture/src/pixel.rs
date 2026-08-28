use porter_math::Vector4;

use crate::ImageFormat;
use crate::pack_unorm8;
use crate::unpack_unorm8;

/// A floating point pixel with 4 components.
#[derive(Debug, Clone, Copy)]
pub struct Pixel {
    /// The red componnent.
    pub r: f32,
    /// The green component.
    pub g: f32,
    /// The blue component.
    pub b: f32,
    /// The alpha component.
    pub a: f32,
}

impl Pixel {
    /// A white pixel.
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    /// A black pixel.
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    /// A transparent pixel.
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Constructs a new pixel with the given components.
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Linearly interpolates between two pixels with the given time.
    #[inline]
    pub const fn lerp(&self, other: Self, time: f32) -> Self {
        Self {
            r: self.r + (other.r - self.r) * time,
            g: self.g + (other.g - self.g) * time,
            b: self.b + (other.b - self.b) * time,
            a: self.a + (other.a - self.a) * time,
        }
    }

    /// Swizzles the order of this pixels components.
    #[inline]
    pub const fn swizzle<const R: usize, const G: usize, const B: usize, const A: usize>(
        &self,
    ) -> Self {
        Self {
            r: self.get(R),
            g: self.get(G),
            b: self.get(B),
            a: self.get(A),
        }
    }

    /// Constructs a new pixel from a unorm8 array `[r, g, b, a]`.
    #[inline]
    pub const fn from_unorm8(value: [u8; 4]) -> Self {
        let r = unpack_unorm8(value[0]);
        let g = unpack_unorm8(value[1]);
        let b = unpack_unorm8(value[2]);
        let a = unpack_unorm8(value[3]);

        Self { r, g, b, a }
    }

    /// Converts this pixel to a unorm8 array `[r, g, b ,a]`.
    #[inline]
    pub const fn to_unorm8(self) -> [u8; 4] {
        let r = pack_unorm8(self.r);
        let g = pack_unorm8(self.g);
        let b = pack_unorm8(self.b);
        let a = pack_unorm8(self.a);

        [r, g, b, a]
    }

    /// Loads a pixel from the provided buffer according to the image format.
    #[inline]
    pub const fn load(format: ImageFormat, buffer: &[u8]) -> Self {
        match format {
            ImageFormat::R8G8B8A8Unorm | ImageFormat::R8G8B8A8UnormSrgb => {
                let r = unpack_unorm8(buffer[0]);
                let g = unpack_unorm8(buffer[1]);
                let b = unpack_unorm8(buffer[2]);
                let a = unpack_unorm8(buffer[3]);

                Self::new(r, g, b, a)
            }
            _ => {
                #[cfg(debug_assertions)]
                panic!("Invalid pixel load format!");
                #[cfg(not(debug_assertions))]
                Self::TRANSPARENT
            }
        }
    }

    /// Gets a component of this pixel by index.
    #[inline]
    const fn get(&self, index: usize) -> f32 {
        match index {
            0 => self.r,
            1 => self.g,
            2 => self.b,
            3 => self.a,
            _ => panic!("Bad index into Pixel!"),
        }
    }
}

impl From<Vector4> for Pixel {
    #[inline]
    fn from(value: Vector4) -> Self {
        Self::new(value.x, value.y, value.z, value.w)
    }
}

impl From<Pixel> for Vector4 {
    #[inline]
    fn from(value: Pixel) -> Self {
        Self::new(value.r, value.g, value.b, value.a)
    }
}
