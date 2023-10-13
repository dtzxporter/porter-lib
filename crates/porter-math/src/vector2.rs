use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

/// A 2d XY vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

assert_eq_size!([u8; 0x8], Vector2);

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
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    #[inline]
    pub fn swizzle<const X: usize, const Y: usize>(&self) -> Self {
        Self {
            x: self[X],
            y: self[Y],
        }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn normalize(&mut self) {
        let length = self.length();

        if length > 0.0 {
            self.x /= length;
            self.y /= length;
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
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    #[inline]
    pub fn lerp(&self, rhs: Self, time: f32) -> Self {
        *self + (rhs - *self) * time
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

impl From<(f32, f32)> for Vector2 {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
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
