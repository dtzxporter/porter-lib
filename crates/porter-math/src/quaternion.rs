use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

use crate::degrees_to_radians;
use crate::Angles;
use crate::Matrix3x3;
use crate::Matrix4x4;
use crate::Vector3;
use crate::Vector4;

/// A 3d XYZW rotation.
#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

assert_eq_size!([u8; 0x10], Quaternion);

impl Quaternion {
    /// Constructs a new quaternion with the given component values.
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Constructs a new identity quaternion.
    #[inline]
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    /// Calculates the length of this quaternion.
    /// `sqrt(x * x + y * y + z * z + w * w)`
    #[inline]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Calculates the length squared of this quaternion.
    /// `x * x + y * y + z * z + w * w`
    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w
    }

    /// Normalizes the quaternion.
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

    /// Returns a quaternion that is normalized.
    #[inline]
    pub fn normalized(&self) -> Self {
        let mut normalize = *self;
        normalize.normalize();
        normalize
    }

    /// Calculates the inverse of this quaternion.
    #[inline]
    pub fn inverse(&self) -> Self {
        let length_squared = self.length_squared();
        let half_length = 1.0 / length_squared;

        Self {
            x: -self.x * half_length,
            y: -self.y * half_length,
            z: -self.z * half_length,
            w: self.w * half_length,
        }
    }

    /// Reverses the byte order of the quaternion.
    #[inline]
    pub fn swap_bytes(self) -> Self {
        Self {
            x: f32::from_bits(self.x.to_bits().swap_bytes()),
            y: f32::from_bits(self.y.to_bits().swap_bytes()),
            z: f32::from_bits(self.z.to_bits().swap_bytes()),
            w: f32::from_bits(self.w.to_bits().swap_bytes()),
        }
    }

    /// Calculates the euler angle rotation of this quaternion.
    #[inline]
    pub fn to_euler(&self, angles: Angles) -> Vector3 {
        self.to_4x4().to_euler(angles)
    }

    /// Calculates the log vector rotation of this quaternion.
    #[inline]
    pub fn to_log_vector(&self) -> Vector3 {
        let sin_half_angle = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();

        if sin_half_angle < f32::EPSILON {
            Vector3::zero()
        } else {
            let fac = sin_half_angle.atan2(self.w) / sin_half_angle;

            Vector3::new(fac * self.x, fac * self.y, fac * self.z)
        }
    }

    /// Converts this quaternion to a rotation matrix.
    #[inline]
    pub fn to_3x3(&self) -> Matrix3x3 {
        let mut matrix = Matrix3x3::new();

        let len_squared = self.length_squared();
        let mut two_div_len = 0.0;

        if len_squared > 0.0 {
            two_div_len = 2.0 / len_squared;
        }

        let xt = self.x * two_div_len;
        let yt = self.y * two_div_len;
        let zt = self.z * two_div_len;

        let wxt = self.w * xt;
        let wyt = self.w * yt;
        let wzt = self.w * zt;

        let xxt = self.x * xt;
        let xyt = self.x * yt;
        let xzt = self.x * zt;

        let yyt = self.y * yt;
        let yzt = self.y * zt;
        let zzt = self.z * zt;

        *matrix.mat_mut::<0, 0>() = 1.0 - (yyt + zzt);
        *matrix.mat_mut::<1, 0>() = xyt - wzt;
        *matrix.mat_mut::<2, 0>() = xzt + wyt;

        *matrix.mat_mut::<0, 1>() = xyt + wzt;
        *matrix.mat_mut::<1, 1>() = 1.0 - (xxt + zzt);
        *matrix.mat_mut::<2, 1>() = yzt - wxt;

        *matrix.mat_mut::<0, 2>() = xzt - wyt;
        *matrix.mat_mut::<1, 2>() = yzt + wxt;
        *matrix.mat_mut::<2, 2>() = 1.0 - (xxt + yyt);

        matrix
    }

    /// Converts this quaternion to a matrix.
    #[inline]
    pub fn to_4x4(&self) -> Matrix4x4 {
        let mut matrix = Matrix4x4::new();

        let len_squared = self.length_squared();
        let mut two_div_len = 0.0;

        if len_squared > 0.0 {
            two_div_len = 2.0 / len_squared;
        }

        let xt = self.x * two_div_len;
        let yt = self.y * two_div_len;
        let zt = self.z * two_div_len;

        let wxt = self.w * xt;
        let wyt = self.w * yt;
        let wzt = self.w * zt;

        let xxt = self.x * xt;
        let xyt = self.x * yt;
        let xzt = self.x * zt;

        let yyt = self.y * yt;
        let yzt = self.y * zt;
        let zzt = self.z * zt;

        *matrix.mat_mut::<0, 0>() = 1.0 - (yyt + zzt);
        *matrix.mat_mut::<1, 0>() = xyt - wzt;
        *matrix.mat_mut::<2, 0>() = xzt + wyt;

        *matrix.mat_mut::<0, 1>() = xyt + wzt;
        *matrix.mat_mut::<1, 1>() = 1.0 - (xxt + zzt);
        *matrix.mat_mut::<2, 1>() = yzt - wxt;

        *matrix.mat_mut::<0, 2>() = xzt - wyt;
        *matrix.mat_mut::<1, 2>() = yzt + wxt;
        *matrix.mat_mut::<2, 2>() = 1.0 - (xxt + yyt);

        matrix
    }

    /// Constructs a new quaternion from the given euler angles.
    #[inline]
    pub fn from_euler(euler: Vector3, angles: Angles) -> Self {
        Self::from_axis_rotation(Vector3::new(0.0, 0.0, 1.0), euler.z, angles)
            * Self::from_axis_rotation(Vector3::new(0.0, 1.0, 0.0), euler.y, angles)
            * Self::from_axis_rotation(Vector3::new(1.0, 0.0, 0.0), euler.x, angles)
    }

    /// Constructs a new quaternion from the given axis rotation.
    #[inline]
    pub fn from_axis_rotation(axis: Vector3, angle: f32, measurment: Angles) -> Self {
        let radians = match measurment {
            Angles::Degrees => degrees_to_radians(angle),
            Angles::Radians => angle,
        };

        let angle_scale = (radians / 2.0).sin();
        let quaternion_scale = (radians / 2.0).cos();

        let angle_result = axis * angle_scale;

        Self {
            x: angle_result.x,
            y: angle_result.y,
            z: angle_result.z,
            w: quaternion_scale,
        }
    }

    /// Constructs a new quaternion from the given log vector rotation.
    #[inline]
    pub fn from_log_vector(vector: Vector3) -> Self {
        let half_angle = vector.length_squared().sqrt();

        if half_angle < f32::EPSILON {
            Self::identity()
        } else {
            let fac = half_angle.sin() / half_angle;

            Self {
                x: fac * vector.x,
                y: fac * vector.y,
                z: fac * vector.z,
                w: half_angle.cos(),
            }
        }
    }
}

impl Default for Quaternion {
    #[inline]
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl From<[f32; 4]> for Quaternion {
    fn from(value: [f32; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}

impl From<Vector4> for Quaternion {
    fn from(value: Vector4) -> Self {
        Self::new(value.x, value.y, value.z, value.w)
    }
}

impl cmp::PartialEq for Quaternion {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < f32::EPSILON
            && (self.y - other.y).abs() < f32::EPSILON
            && (self.z - other.z).abs() < f32::EPSILON
            && (self.w - other.w).abs() < f32::EPSILON
    }
}

impl ops::Index<usize> for Quaternion {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("Bad index into Quaternion!"),
        }
    }
}

impl ops::IndexMut<usize> for Quaternion {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("Bad index into Quaternion!"),
        }
    }
}

impl ops::Add<Quaternion> for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn add(self, rhs: Quaternion) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.y + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl ops::Sub<Quaternion> for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn sub(self, rhs: Quaternion) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.y - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl ops::Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn mul(self, rhs: Quaternion) -> Self::Output {
        Self {
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
        }
    }
}

impl ops::Mul<f32> for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl ops::Neg for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl ops::Not for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn not(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }
}
