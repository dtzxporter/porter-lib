use std::cmp;
use std::ops;

use static_assertions::assert_eq_size;

use crate::degrees_to_radians;
use crate::Angles;
use crate::Matrix4x4;
use crate::Vector3;

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
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
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

    #[inline]
    pub fn euler_angles(&self, measurment: Angles) -> Vector3 {
        self.matrix4x4().euler_angles(measurment)
    }

    #[inline]
    pub fn matrix4x4(&self) -> Matrix4x4 {
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

    #[inline]
    pub fn from_euler_angles(x: f32, y: f32, z: f32, measurment: Angles) -> Self {
        Self::from_axis_rotation(Vector3::new(1.0, 0.0, 0.0), x, measurment)
            * Self::from_axis_rotation(Vector3::new(0.0, 1.0, 0.0), y, measurment)
            * Self::from_axis_rotation(Vector3::new(0.0, 0.0, 1.0), z, measurment)
    }

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
