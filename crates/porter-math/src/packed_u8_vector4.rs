use static_assertions::assert_eq_size;

use crate::Vector2;
use crate::Vector3;
use crate::Vector4;

/// A 3d XYZW unit vector packed to unsigned bytes in the range [-1.0, 1.0].
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PackedU8Vector4 {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub w: u8,
}

assert_eq_size!([u8; 0x4], PackedU8Vector4);

impl PackedU8Vector4 {
    #[inline]
    pub const fn new(x: u8, y: u8, z: u8, w: u8) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn vector4(self) -> Vector4 {
        self.into()
    }

    #[inline]
    pub fn vector3(self) -> Vector3 {
        Vector4::from(self).into()
    }

    #[inline]
    pub fn vector2(self) -> Vector2 {
        Vector3::from(Vector4::from(self)).into()
    }
}

impl From<PackedU8Vector4> for Vector4 {
    fn from(value: PackedU8Vector4) -> Self {
        Self::new(
            (value.x as f32 / i8::MAX as f32) - 1.0,
            (value.y as f32 / i8::MAX as f32) - 1.0,
            (value.z as f32 / i8::MAX as f32) - 1.0,
            (value.w as f32 / i8::MAX as f32) - 1.0,
        )
    }
}
