use static_assertions::assert_eq_size;

use crate::Vector2;
use crate::Vector3;
use crate::Vector4;

/// A 3d XYZ(W) packed vector with 10 bits for XYZ and 2 bits for W.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Packed102Vector4 {
    pub packed: u32,
}

assert_eq_size!([u8; 0x4], Packed102Vector4);

impl Packed102Vector4 {
    #[inline]
    pub const fn new(packed: u32) -> Self {
        Self { packed }
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

impl From<Packed102Vector4> for Vector4 {
    fn from(value: Packed102Vector4) -> Self {
        let x = value.packed & 0x3FF;
        let y = (value.packed >> 10) & 0x3FF;
        let z = (value.packed >> 20) & 0x3FF;
        let w = (value.packed >> 30) & 0x3;

        Self::new(
            x as f32 / 1023.0,
            y as f32 / 1023.0,
            z as f32 / 1023.0,
            w as f32 / 3.0,
        )
    }
}
