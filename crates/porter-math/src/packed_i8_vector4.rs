use static_assertions::assert_eq_size;

use crate::Vector2;
use crate::Vector3;
use crate::Vector4;

/// A 3d XYZW unit vector packed to signed bytes in the range [-1.0, 1.0].
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PackedI8Vector4 {
    pub x: i8,
    pub y: i8,
    pub z: i8,
    pub w: i8,
}

assert_eq_size!([u8; 0x4], PackedI8Vector4);

impl PackedI8Vector4 {
    #[inline]
    pub const fn new(x: i8, y: i8, z: i8, w: i8) -> Self {
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
        Vector4::from(self).into()
    }
}

impl From<PackedI8Vector4> for Vector4 {
    fn from(value: PackedI8Vector4) -> Self {
        Self::new(
            value.x as f32 / i8::MAX as f32,
            value.y as f32 / i8::MAX as f32,
            value.z as f32 / i8::MAX as f32,
            value.w as f32 / i8::MAX as f32,
        )
    }
}
