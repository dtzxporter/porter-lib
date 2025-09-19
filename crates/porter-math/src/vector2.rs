use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Vector3;
use crate::Vector4;

/// A 2d XY vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

assert_eq_size!([u8; 0x8], Vector2);

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
            }
        }
    };
    ($op:ty, $for:ty, $name:ident, $operand:tt) => {
        impl $op for $for {
            #[inline]
            fn $name(&mut self, rhs: Self) {
                self.x $operand rhs.x;
                self.y $operand rhs.y;
            }
        }
    };
}

impl Vector2 {
    /// Constructs a new vector with the given component values.
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Constructs a new vector where all components are `0.0`.
    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Constructs a new vector where all components are `1.0`.
    #[inline]
    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    /// Construct a new vector where all components are `value`.
    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self { x: value, y: value }
    }

    /// Swizzles the order of the vectors components.
    #[inline]
    pub fn swizzle<const X: usize, const Y: usize>(&self) -> Self {
        Self {
            x: self[X],
            y: self[Y],
        }
    }

    /// Calculates the length of this vector.
    /// `sqrt(x * x + y * y)`
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Calculates the length squared of this vector.
    /// `x * x + y * y`
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Normalizes the vector.
    #[inline]
    pub fn normalize(&mut self) {
        let length = self.length();

        if length > f32::EPSILON {
            self.x /= length;
            self.y /= length;
        }
    }

    /// Returns a vector that is normalized.
    #[inline]
    pub fn normalized(&self) -> Self {
        let mut normalize = *self;
        normalize.normalize();
        normalize
    }

    /// Calculates the dot product of the two vectors.
    /// `(x * rhs.x) + (y * rhs.y)`
    #[inline]
    pub fn dot(&self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
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
        }
    }

    /// Converts this vector to an octahedron vector.
    #[inline]
    pub fn to_octahedron(self, signed: bool) -> Vector3 {
        let xy = if signed { self * 2.0 - 1.0 } else { self };

        let mut normal = Vector3::new(xy.x, xy.y, 1.0 - (xy.x.abs()) - (xy.y.abs()));
        let avg = (-normal.z).clamp(0.0, 1.0);

        normal.x += if normal.x >= 0.0 { -avg } else { avg };
        normal.y += if normal.y >= 0.0 { -avg } else { avg };

        normal.normalized()
    }

    /// Returns a vector with any components that are `NaN` set to `0.0`.
    #[inline]
    pub fn nan_to_zero(self) -> Self {
        Self {
            x: if self.x.is_nan() { 0.0 } else { self.x },
            y: if self.y.is_nan() { 0.0 } else { self.y },
        }
    }

    /// Returns `true` if the vector is normalized having a length of `1.0`.
    #[inline]
    pub fn is_normalized(&self) -> bool {
        (self.length_squared().abs() - 1.0) <= 2e-4
    }
}

impl cmp::PartialEq for Vector2 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f32::EPSILON && (self.y - other.y).abs() < f32::EPSILON
    }
}

impl ops::Index<usize> for Vector2 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Bad index into Vector2!"),
        }
    }
}

impl ops::IndexMut<usize> for Vector2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => panic!("Bad index into Vector2!"),
        }
    }
}

impl From<[f32; 2]> for Vector2 {
    fn from(value: [f32; 2]) -> Self {
        Self::new(value[0], value[1])
    }
}

impl From<(f32, f32)> for Vector2 {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Vector3> for Vector2 {
    fn from(value: Vector3) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Vector4> for Vector2 {
    fn from(value: Vector4) -> Self {
        Self::new(value.x, value.y)
    }
}

impl ops::Neg for Vector2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

// Vector2 -> Vector2 ops
impl_op_routine!(ops::Add<Vector2>, Vector2, add, +);
impl_op_routine!(ops::Sub<Vector2>, Vector2, sub, -);
impl_op_routine!(ops::Div<Vector2>, Vector2, div, /);
impl_op_routine!(ops::Mul<Vector2>, Vector2, mul, *);
// Vector2 -> f32 ops
impl_op_routine!(f32, ops::Add<f32>, Vector2, add, +);
impl_op_routine!(f32, ops::Sub<f32>, Vector2, sub, -);
impl_op_routine!(f32, ops::Div<f32>, Vector2, div, /);
impl_op_routine!(f32, ops::Mul<f32>, Vector2, mul, *);

// Vector3 -> Vector3 ops
impl_op_assign_routine!(ops::AddAssign<Vector2>, Vector2, add_assign, +=);
impl_op_assign_routine!(ops::SubAssign<Vector2>, Vector2, sub_assign, -=);
impl_op_assign_routine!(ops::DivAssign<Vector2>, Vector2, div_assign, /=);
impl_op_assign_routine!(ops::MulAssign<Vector2>, Vector2, mul_assign, *=);
// Vector3 -> f32 ops
impl_op_assign_routine!(f32, ops::AddAssign<f32>, Vector2, add_assign, +=);
impl_op_assign_routine!(f32, ops::SubAssign<f32>, Vector2, sub_assign, -=);
impl_op_assign_routine!(f32, ops::DivAssign<f32>, Vector2, div_assign, /=);
impl_op_assign_routine!(f32, ops::MulAssign<f32>, Vector2, mul_assign, *=);
