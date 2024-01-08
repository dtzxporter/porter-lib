use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Matrix4x4;

/// A 3d XYZW vector.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

assert_eq_size!([u8; 0x10], Vector4);

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
                    w: self.w $operand rhs,
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
                    w: self.w $operand rhs.w,
                }
            }
        }
    };
}

macro_rules! impl_op_assign_routine {
    ($structt:ty, $op:ty, $for:ty, $name:ident, $operand:tt) => {
        impl $op for $for {
            #[inline]
            fn $name(&mut self, rhs: $structt) {
                self.x $operand rhs;
                self.y $operand rhs;
                self.z $operand rhs;
                self.w $operand rhs;
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
                self.w $operand rhs.w;
            }
        }
    };
}

impl Vector4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    #[inline]
    pub const fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            w: 1.0,
        }
    }

    #[inline]
    pub fn swizzle<const X: usize, const Y: usize, const Z: usize, const W: usize>(&self) -> Self {
        Self {
            x: self[X],
            y: self[Y],
            z: self[Z],
            w: self[W],
        }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    #[inline]
    pub fn normalize(&mut self) {
        let length = self.length();

        if length > 0.0 {
            self.x /= length;
            self.y /= length;
            self.z /= length;
            self.w /= length;
        }
    }

    #[inline]
    pub fn normalized(&self) -> Self {
        let mut normalize = *self;
        normalize.normalize();
        normalize
    }

    #[inline]
    pub fn dot(&self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    #[inline]
    pub fn lerp(&self, rhs: Self, time: f32) -> Self {
        *self + (rhs - *self) * time
    }

    #[inline]
    pub fn swap_bytes(self) -> Self {
        Self {
            x: f32::from_bits(self.x.to_bits().swap_bytes()),
            y: f32::from_bits(self.y.to_bits().swap_bytes()),
            z: f32::from_bits(self.z.to_bits().swap_bytes()),
            w: f32::from_bits(self.w.to_bits().swap_bytes()),
        }
    }

    #[inline]
    pub fn transform(&self, value: &Matrix4x4) -> Self {
        Self {
            x: (self.x * value.mat::<0, 0>())
                + (self.y * value.mat::<1, 0>())
                + (self.z * value.mat::<2, 0>())
                + (self.w * value.mat::<3, 0>()),
            y: (self.x * value.mat::<0, 1>())
                + (self.y * value.mat::<1, 1>())
                + (self.z * value.mat::<2, 1>())
                + (self.w * value.mat::<3, 1>()),
            z: (self.x * value.mat::<0, 2>())
                + (self.y * value.mat::<1, 2>())
                + (self.z * value.mat::<2, 2>())
                + (self.w * value.mat::<3, 2>()),
            w: (self.x * value.mat::<0, 3>())
                + (self.y * value.mat::<1, 3>())
                + (self.z * value.mat::<2, 3>())
                + (self.w * value.mat::<3, 3>()),
        }
    }
}

impl cmp::PartialEq for Vector4 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f32::EPSILON
            && (self.y - other.y).abs() < f32::EPSILON
            && (self.z - other.z).abs() < f32::EPSILON
            && (self.w - other.w).abs() < f32::EPSILON
    }
}

impl ops::Index<usize> for Vector4 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Bad index into Vector4!"),
        }
    }
}

impl ops::IndexMut<usize> for Vector4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Bad index into Vector4!"),
        }
    }
}

impl From<(f32, f32, f32, f32)> for Vector4 {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

// Vector4 -> Vector4 ops
impl_op_routine!(ops::Add<Vector4>, Vector4, add, +);
impl_op_routine!(ops::Sub<Vector4>, Vector4, sub, -);
impl_op_routine!(ops::Div<Vector4>, Vector4, div, /);
impl_op_routine!(ops::Mul<Vector4>, Vector4, mul, *);
// Vector4 -> f32 ops
impl_op_routine!(f32, ops::Add<f32>, Vector4, add, +);
impl_op_routine!(f32, ops::Sub<f32>, Vector4, sub, -);
impl_op_routine!(f32, ops::Div<f32>, Vector4, div, /);
impl_op_routine!(f32, ops::Mul<f32>, Vector4, mul, *);

// Vector4 -> Vector4 ops
impl_op_assign_routine!(ops::AddAssign<Vector4>, Vector4, add_assign, +=);
impl_op_assign_routine!(ops::SubAssign<Vector4>, Vector4, sub_assign, -=);
impl_op_assign_routine!(ops::DivAssign<Vector4>, Vector4, div_assign, /=);
impl_op_assign_routine!(ops::MulAssign<Vector4>, Vector4, mul_assign, *=);
// Vector4 -> f32 ops
impl_op_assign_routine!(f32, ops::AddAssign<f32>, Vector4, add_assign, +=);
impl_op_assign_routine!(f32, ops::SubAssign<f32>, Vector4, sub_assign, -=);
impl_op_assign_routine!(f32, ops::DivAssign<f32>, Vector4, div_assign, /=);
impl_op_assign_routine!(f32, ops::MulAssign<f32>, Vector4, mul_assign, *=);
