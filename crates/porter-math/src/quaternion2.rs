use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Angles;
use crate::Matrix3x3;
use crate::Matrix4x4;
use crate::Quaternion;
use crate::Vector2;
use crate::Vector3;

/// A 3d ZW rotation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Quaternion2 {
    pub z: f32,
    pub w: f32,
}

assert_eq_size!([u8; 0x8], Quaternion2);

impl Quaternion2 {
    /// Constructs a new quaternion with the given component values.
    #[inline]
    pub fn new(z: f32, w: f32) -> Self {
        Self { z, w }
    }

    /// Constructs a new identity quaternion.
    #[inline]
    pub fn identity() -> Self {
        Self { z: 0.0, w: 1.0 }
    }

    /// Calculates the length of this quaternion.
    /// `sqrt(z * z + w * w)`
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Calculates the length squared of this quaternion.
    /// `z * z + w * w`
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.z * self.z + self.w * self.w
    }

    /// Normalizes the quaternion.
    #[inline]
    pub fn normalize(&mut self) {
        let length = self.length();

        if length > 0.0 {
            self.z /= length;
            self.w /= length;
        }
    }

    /// Returns a quaternion that is normalized.
    #[inline]
    pub fn normalized(&self) -> Self {
        let mut normalize = *self;
        normalize.normalize();
        normalize
    }

    /// Calculates the dot product of the two quaternions.
    /// `(z * rhs.z) + (w * rhs.w)`
    #[inline]
    pub fn dot(&self, rhs: Self) -> f32 {
        (self.z * rhs.z) + (self.w * rhs.w)
    }

    /// Spherical linear interpolation between two quaternions.
    #[inline]
    pub fn slerp(&self, rhs: Self, time: f32) -> Self {
        let rhs = rhs.to_quat4();
        let lhs = self.to_quat4();

        lhs.slerp(rhs, time).to_quat2()
    }

    /// Reverses the byte order of the quaternion.
    #[inline]
    pub fn swap_bytes(self) -> Self {
        Self {
            z: f32::from_bits(self.z.to_bits().swap_bytes()),
            w: f32::from_bits(self.w.to_bits().swap_bytes()),
        }
    }

    /// Calculates the euler angle rotation of this quaternion.
    #[inline]
    pub fn to_euler(&self, angles: Angles) -> Vector3 {
        self.to_quat4().to_4x4().to_euler(angles)
    }

    /// Converts this quaternion to a rotation matrix.
    #[inline]
    pub fn to_3x3(&self) -> Matrix3x3 {
        self.to_quat4().to_3x3()
    }

    /// Converts this quaternion to a matrix.
    #[inline]
    pub fn to_4x4(&self) -> Matrix4x4 {
        self.to_quat4().to_4x4()
    }

    /// Converts this ZW quaternion to a XYZW quaternion.
    #[inline]
    pub fn to_quat4(self) -> Quaternion {
        Quaternion::from(self)
    }

    /// Returns `true` if the quaternion is normalized having a length of `1.0`.
    #[inline]
    pub fn is_normalized(&self) -> bool {
        (self.length_squared().abs() - 1.0) <= 2e-4
    }
}

impl Default for Quaternion2 {
    #[inline]
    fn default() -> Self {
        Self { z: 0.0, w: 1.0 }
    }
}

impl From<[f32; 2]> for Quaternion2 {
    fn from(value: [f32; 2]) -> Self {
        Self {
            z: value[0],
            w: value[1],
        }
    }
}

impl From<Vector2> for Quaternion2 {
    fn from(value: Vector2) -> Self {
        Self {
            z: value.x,
            w: value.y,
        }
    }
}

impl From<Quaternion> for Quaternion2 {
    fn from(value: Quaternion) -> Self {
        Self {
            z: value.z,
            w: value.w,
        }
    }
}

impl cmp::PartialEq for Quaternion2 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.z - other.z).abs() < f32::EPSILON && (self.w - other.w).abs() < f32::EPSILON
    }
}

impl ops::Index<usize> for Quaternion2 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.z,
            1 => &self.w,
            _ => panic!("Bad index into Quaternion2!"),
        }
    }
}

impl ops::IndexMut<usize> for Quaternion2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.z,
            1 => &mut self.w,
            _ => panic!("Bad index into Quaternion2!"),
        }
    }
}
