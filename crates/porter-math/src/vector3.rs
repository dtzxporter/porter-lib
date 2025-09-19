use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Matrix4x4;
use crate::Quaternion;
use crate::Vector4;

/// A 3d XYZ vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

assert_eq_size!([u8; 0xC], Vector3);

/// Utility to implement the regular op traits.
macro_rules! impl_op_routine {
    ($structt:ty, $op:ty, $for:ty, $name:ident, $operand:tt) => {
        impl $op for $for {
            type Output = $for;

            #[inline]
            fn $name(self, rhs: $structt) -> Self::Output {
                Self {
                    x: self.x $operand rhs,
                    y: self.y $operand rhs,
                    z: self.z $operand rhs,
                }
            }
        }
    };
    ($op:ty, $for:ty, $name:ident, $operand:tt) => {
        impl $op for $for {
            type Output = $for;

            #[inline]
            fn $name(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x $operand rhs.x,
                    y: self.y $operand rhs.y,
                    z: self.z $operand rhs.z,
                }
            }
        }
    };
}

/// Utility to implement the assignment op traits.
macro_rules! impl_op_assign_routine {
    ($structt:ty, $op:ty, $for:ty, $name:ident, $operand:tt) => {
        impl $op for $for {
            #[inline]
            fn $name(&mut self, rhs: $structt) {
                self.x $operand rhs;
                self.y $operand rhs;
                self.z $operand rhs;
            }
        }
    };
    ($op:ty, $for:ty, $name:ident, $operand:tt) => {
        impl $op for $for {
            #[inline]
            fn $name(&mut self, rhs: Self) {
                self.x $operand rhs.x;
                self.y $operand rhs.y;
                self.z $operand rhs.z;
            }
        }
    };
}

impl Vector3 {
    /// Constructs a new vector with the given component values.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Constructs a new vector where all components are `0.0`.
    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Constructs a new vector where all components are `1.0`.
    #[inline]
    pub const fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    /// Construct a new vector where all components are `value`.
    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }

    /// Swizzles the order of the vectors components.
    #[inline]
    pub fn swizzle<const X: usize, const Y: usize, const Z: usize>(&self) -> Self {
        Self {
            x: self[X],
            y: self[Y],
            z: self[Z],
        }
    }

    /// Calculates the length of this vector.
    /// `sqrt(x * x + y * y + z * z)`
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Calculates the length squared of this vector.
    /// `x * x + y * y + z * z`
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Normalizes the vector.
    #[inline]
    pub fn normalize(&mut self) {
        let length = self.length();

        if length > f32::EPSILON {
            self.x /= length;
            self.y /= length;
            self.z /= length;
        }
    }

    /// Returns a vector that is normalized.
    #[inline]
    pub fn normalized(&self) -> Self {
        let mut normalize = *self;
        normalize.normalize();
        normalize
    }

    /// Calculates the cross product of the two vectors.
    #[inline]
    pub fn cross(&self, rhs: Self) -> Self {
        Self {
            x: (self.y * rhs.z) - (self.z * rhs.y),
            y: (self.z * rhs.x) - (self.x * rhs.z),
            z: (self.x * rhs.y) - (self.y * rhs.x),
        }
    }

    /// Calculates the dot product of the two vectors.
    /// `(x * rhs.x) + (y * rhs.y) + (z * rhs.z)`
    #[inline]
    pub fn dot(&self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    /// Linearly interpolates between two vectors with the given time.
    #[inline]
    pub fn lerp(&self, rhs: Self, time: f32) -> Self {
        *self + (rhs - *self) * time
    }

    /// Reverses the byte order of the vector.
    #[inline]
    pub fn swap_bytes(self) -> Self {
        Self {
            x: f32::from_bits(self.x.to_bits().swap_bytes()),
            y: f32::from_bits(self.y.to_bits().swap_bytes()),
            z: f32::from_bits(self.z.to_bits().swap_bytes()),
        }
    }

    /// Transforms this vector with the given matrix.
    #[inline]
    pub fn transform(&self, value: &Matrix4x4) -> Self {
        Self {
            x: (self.x * value.mat::<0, 0>())
                + (self.y * value.mat::<1, 0>())
                + (self.z * value.mat::<2, 0>())
                + value.mat::<3, 0>(),
            y: (self.x * value.mat::<0, 1>())
                + (self.y * value.mat::<1, 1>())
                + (self.z * value.mat::<2, 1>())
                + value.mat::<3, 1>(),
            z: (self.x * value.mat::<0, 2>())
                + (self.y * value.mat::<1, 2>())
                + (self.z * value.mat::<2, 2>())
                + value.mat::<3, 2>(),
        }
    }

    /// Rotates this vector with the given quaternion.
    #[inline]
    pub fn rotate(&self, value: Quaternion) -> Self {
        let u = Vector3::new(value.x, value.y, value.z);
        let s = value.w;

        let uv = u.cross(*self);
        let uuv = u.cross(uv);

        *self + (uv * (2.0 * s)) + (uuv * 2.0)
    }

    /// Returns a vector with any components that are `NaN` set to `0.0`.
    #[inline]
    pub fn nan_to_zero(self) -> Self {
        Self {
            x: if self.x.is_nan() { 0.0 } else { self.x },
            y: if self.y.is_nan() { 0.0 } else { self.y },
            z: if self.z.is_nan() { 0.0 } else { self.z },
        }
    }

    /// Returns any unit vector that is orthogonal to this unit vector.
    #[inline]
    pub fn orthonormal_vector(&self) -> Self {
        debug_assert!(self.is_normalized());

        let sign = self.z.signum();
        let a = -1.0 / (sign + self.z);
        let b = self.x * self.y * a;

        Self {
            x: b,
            y: sign + self.y * self.y * a,
            z: -self.y,
        }
    }

    /// Returns the angle (in radians) between two unit vectors in the range `[0, +π]`.
    #[inline]
    pub fn angle_between(&self, rhs: Self) -> f32 {
        debug_assert!(self.is_normalized());
        debug_assert!(rhs.is_normalized());

        let cosine = self.dot(rhs);

        if cosine >= 1.0 {
            0.0
        } else if cosine <= -1.0 {
            std::f32::consts::PI
        } else {
            cosine.acos()
        }
    }

    /// Return `true` if the unit vector is parallel (same or opposite direction) to the given unit vector.
    #[inline]
    pub fn is_parallel(&self, rhs: Self) -> bool {
        debug_assert!(self.is_normalized());
        debug_assert!(rhs.is_normalized());

        self.dot(rhs).abs() >= 1.0 - f32::EPSILON
    }

    /// Returns `true` if the vector is normalized having a length of `1.0`.
    #[inline]
    pub fn is_normalized(&self) -> bool {
        (self.length_squared().abs() - 1.0) <= 2e-4
    }
}

impl cmp::PartialEq for Vector3 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f32::EPSILON
            && (self.y - other.y).abs() < f32::EPSILON
            && (self.z - other.z).abs() < f32::EPSILON
    }
}

impl ops::Index<usize> for Vector3 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Bad index into Vector3!"),
        }
    }
}

impl ops::IndexMut<usize> for Vector3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Bad index into Vector3!"),
        }
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from(value: [f32; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }
}

impl From<(f32, f32, f32)> for Vector3 {
    fn from(value: (f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl From<Vector4> for Vector3 {
    fn from(value: Vector4) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl ops::Neg for Vector3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// Vector3 -> Vector3 ops
impl_op_routine!(ops::Add<Vector3>, Vector3, add, +);
impl_op_routine!(ops::Sub<Vector3>, Vector3, sub, -);
impl_op_routine!(ops::Div<Vector3>, Vector3, div, /);
impl_op_routine!(ops::Mul<Vector3>, Vector3, mul, *);
// Vector3 -> f32 ops
impl_op_routine!(f32, ops::Add<f32>, Vector3, add, +);
impl_op_routine!(f32, ops::Sub<f32>, Vector3, sub, -);
impl_op_routine!(f32, ops::Div<f32>, Vector3, div, /);
impl_op_routine!(f32, ops::Mul<f32>, Vector3, mul, *);

// Vector3 -> Vector3 ops
impl_op_assign_routine!(ops::AddAssign<Vector3>, Vector3, add_assign, +=);
impl_op_assign_routine!(ops::SubAssign<Vector3>, Vector3, sub_assign, -=);
impl_op_assign_routine!(ops::DivAssign<Vector3>, Vector3, div_assign, /=);
impl_op_assign_routine!(ops::MulAssign<Vector3>, Vector3, mul_assign, *=);
// Vector3 -> f32 ops
impl_op_assign_routine!(f32, ops::AddAssign<f32>, Vector3, add_assign, +=);
impl_op_assign_routine!(f32, ops::SubAssign<f32>, Vector3, sub_assign, -=);
impl_op_assign_routine!(f32, ops::DivAssign<f32>, Vector3, div_assign, /=);
impl_op_assign_routine!(f32, ops::MulAssign<f32>, Vector3, mul_assign, *=);
