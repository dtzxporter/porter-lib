use std::cmp;
use std::ops;

use porter_macros::assert_size;

use crate::Matrix4x4;
use crate::Quaternion;

/// A 3d XYZW vector.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

assert_size!(Vector4, 16);

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

/// Utility to implement the assignment op traits.
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
    /// Constructs a new vector with the given component values.
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Constructs a new vector where all components are `0.0`.
    #[inline]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        }
    }

    /// Constructs a new vector where all components are `1.0`.
    #[inline]
    pub const fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            w: 1.0,
        }
    }

    /// Construct a new vector where all components are `value`.
    #[inline]
    pub const fn splat(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
            w: value,
        }
    }

    /// Returns the minimum of two vectors.
    #[inline]
    pub const fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
            w: self.w.min(other.w),
        }
    }

    /// Returns the maximum of two vectors.
    #[inline]
    pub const fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
            w: self.w.max(other.w),
        }
    }

    /// Swizzles the order of the vectors components.
    #[inline]
    pub fn swizzle<const X: usize, const Y: usize, const Z: usize, const W: usize>(&self) -> Self {
        Self {
            x: self[X],
            y: self[Y],
            z: self[Z],
            w: self[W],
        }
    }

    /// Returns a vector mask containing the result of a `==` comparison for each element of `self` and `other`.
    #[inline]
    pub const fn cmpeq(&self, other: Self) -> Self {
        Self {
            x: ((self.x - other.x).abs() < f32::EPSILON) as i32 as f32,
            y: ((self.y - other.y).abs() < f32::EPSILON) as i32 as f32,
            z: ((self.z - other.z).abs() < f32::EPSILON) as i32 as f32,
            w: ((self.w - other.w).abs() < f32::EPSILON) as i32 as f32,
        }
    }

    /// Returns a vector mask containing the result of a `!=` comparison for each element of `self` and `other`.
    #[inline]
    pub const fn cmpne(&self, other: Self) -> Self {
        Self {
            x: ((self.x - other.x).abs() >= f32::EPSILON) as i32 as f32,
            y: ((self.y - other.y).abs() >= f32::EPSILON) as i32 as f32,
            z: ((self.z - other.z).abs() >= f32::EPSILON) as i32 as f32,
            w: ((self.w - other.w).abs() >= f32::EPSILON) as i32 as f32,
        }
    }

    /// Returns a vector from the elements in `self` if the mask is true or `other` if false.
    #[inline]
    pub const fn select(&self, mask: Self, other: Self) -> Self {
        Self {
            x: if mask.x != 0.0 { self.x } else { other.x },
            y: if mask.y != 0.0 { self.y } else { other.y },
            z: if mask.z != 0.0 { self.z } else { other.z },
            w: if mask.w != 0.0 { self.w } else { other.w },
        }
    }

    /// Calculates the length of this vector.
    /// `sqrt(x * x + y * y + z * z + w * w)`
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Calculates the length squared of this vector.
    /// `x * x + y * y + z * z + w * w`
    #[inline]
    pub const fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Normalizes the vector.
    #[inline]
    pub fn normalize(&mut self) {
        let length = self.length();

        if length > f32::EPSILON {
            self.x /= length;
            self.y /= length;
            self.z /= length;
            self.w /= length;
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
    /// `(x * rhs.x) + (y * rhs.y) + (z * rhs.z) + (w * rhs.w)`
    #[inline]
    pub const fn dot(&self, rhs: Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    /// Linearly interpolates between two vectors with the given time.
    #[inline]
    pub fn lerp(&self, rhs: Self, time: f32) -> Self {
        *self + (rhs - *self) * time
    }

    /// Reverses the byte order of the vector.
    #[inline]
    pub const fn swap_bytes(self) -> Self {
        Self {
            x: f32::from_bits(self.x.to_bits().swap_bytes()),
            y: f32::from_bits(self.y.to_bits().swap_bytes()),
            z: f32::from_bits(self.z.to_bits().swap_bytes()),
            w: f32::from_bits(self.w.to_bits().swap_bytes()),
        }
    }

    /// Creates a native endian vector value from its representation as a byte array in in big endian.
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        let [b1, b2, b3, b4, b5, b6, b7, b8, ..] = bytes;
        let [.., b9, b10, b11, b12, b13, b14, b15, b16] = bytes;

        Self {
            x: f32::from_be_bytes([b1, b2, b3, b4]),
            y: f32::from_be_bytes([b5, b6, b7, b8]),
            z: f32::from_be_bytes([b9, b10, b11, b12]),
            w: f32::from_be_bytes([b13, b14, b15, b16]),
        }
    }

    /// Creates a native endian vector value from its representation as a byte array in little endian.
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        let [b1, b2, b3, b4, b5, b6, b7, b8, ..] = bytes;
        let [.., b9, b10, b11, b12, b13, b14, b15, b16] = bytes;

        Self {
            x: f32::from_le_bytes([b1, b2, b3, b4]),
            y: f32::from_le_bytes([b5, b6, b7, b8]),
            z: f32::from_le_bytes([b9, b10, b11, b12]),
            w: f32::from_le_bytes([b13, b14, b15, b16]),
        }
    }

    /// Creates a native endian vector value from its representation as a byte array in native endianness.
    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        let [b1, b2, b3, b4, b5, b6, b7, b8, ..] = bytes;
        let [.., b9, b10, b11, b12, b13, b14, b15, b16] = bytes;

        Self {
            x: f32::from_ne_bytes([b1, b2, b3, b4]),
            y: f32::from_ne_bytes([b5, b6, b7, b8]),
            z: f32::from_ne_bytes([b9, b10, b11, b12]),
            w: f32::from_ne_bytes([b13, b14, b15, b16]),
        }
    }

    /// Returns the memory representation of this vector as a byte array in big-endian (network) byte order.
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        let [b1, b2, b3, b4] = self.x.to_be_bytes();
        let [b5, b6, b7, b8] = self.y.to_be_bytes();
        let [b9, b10, b11, b12] = self.z.to_be_bytes();
        let [b13, b14, b15, b16] = self.w.to_be_bytes();

        [
            b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16,
        ]
    }

    /// Returns the memory representation of this vector as a byte array in little-endian byte order.
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        let [b1, b2, b3, b4] = self.x.to_le_bytes();
        let [b5, b6, b7, b8] = self.y.to_le_bytes();
        let [b9, b10, b11, b12] = self.z.to_le_bytes();
        let [b13, b14, b15, b16] = self.w.to_le_bytes();

        [
            b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16,
        ]
    }

    /// Returns the memory representation of this vector as a byte array in native byte order.
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; size_of::<Self>()] {
        let [b1, b2, b3, b4] = self.x.to_ne_bytes();
        let [b5, b6, b7, b8] = self.y.to_ne_bytes();
        let [b9, b10, b11, b12] = self.z.to_ne_bytes();
        let [b13, b14, b15, b16] = self.w.to_ne_bytes();

        [
            b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15, b16,
        ]
    }

    /// Creates a vector from its representation as a packed 10_2 unorm byte array.
    #[inline]
    pub const fn from_packed_10_2_unorm(value: [u8; 4]) -> Self {
        let packed = u32::from_le_bytes(value);

        let x = packed & 0x3FF;
        let y = (packed >> 10) & 0x3FF;
        let z = (packed >> 20) & 0x3FF;
        let w = (packed >> 30) & 0x3;

        Self::new(
            x as f32 / 1023.0,
            y as f32 / 1023.0,
            z as f32 / 1023.0,
            w as f32 / 3.0,
        )
    }

    /// Creates a vector from its representation as a packed u8 unorm byte array.
    #[inline]
    pub const fn from_packed_u8_unorm(value: [u8; 4]) -> Self {
        Self::new(
            value[0] as f32 / u8::MAX as f32,
            value[1] as f32 / u8::MAX as f32,
            value[2] as f32 / u8::MAX as f32,
            value[3] as f32 / u8::MAX as f32,
        )
    }

    /// Creates a vector from its representation as a packed u8 snorm byte array.
    #[inline]
    pub const fn from_packed_u8_snorm(value: [u8; 4]) -> Self {
        Self::new(
            ((value[0] as f32 / u8::MAX as f32) * 2.0) - 1.0,
            ((value[1] as f32 / u8::MAX as f32) * 2.0) - 1.0,
            ((value[2] as f32 / u8::MAX as f32) * 2.0) - 1.0,
            ((value[3] as f32 / u8::MAX as f32) * 2.0) - 1.0,
        )
    }

    /// Creates a vector from its representation as a packed i8 snorm byte array.
    #[inline]
    pub const fn from_packed_i8_snorm(value: [i8; 4]) -> Self {
        Self::new(
            value[0] as f32 / i8::MAX as f32,
            value[1] as f32 / i8::MAX as f32,
            value[2] as f32 / i8::MAX as f32,
            value[3] as f32 / i8::MAX as f32,
        )
    }

    /// Transforms this vector with the given matrix.
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

    /// Returns `true` if the vector is normalized having a length of `1.0`.
    #[inline]
    pub const fn is_normalized(&self) -> bool {
        (self.length_squared().abs() - 1.0) <= 2e-4
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

impl From<[f32; 4]> for Vector4 {
    fn from(value: [f32; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

impl From<(f32, f32, f32, f32)> for Vector4 {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl From<Quaternion> for Vector4 {
    fn from(value: Quaternion) -> Self {
        Self::new(value.x, value.y, value.z, value.w)
    }
}

impl ops::Neg for Vector4 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
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
