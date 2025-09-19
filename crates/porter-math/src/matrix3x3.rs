use std::cmp;
use std::fmt;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Angles;
use crate::Matrix4x4;
use crate::Quaternion;
use crate::Vector3;
use crate::radians_to_degrees;

/// Represents a 3x3 matrix in column major order.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Matrix3x3 {
    data: [f32; 9],
}

assert_eq_size!([u8; 36], Matrix3x3);

impl Matrix3x3 {
    /// Constructs a new identity matrix.
    #[inline]
    pub const fn new() -> Self {
        let mut data: [f32; 9] = [0.0; 9];

        data[0] = 1.0;
        data[1] = 0.0;
        data[2] = 0.0;

        data[3] = 0.0;
        data[4] = 1.0;
        data[5] = 0.0;

        data[6] = 0.0;
        data[7] = 0.0;
        data[8] = 1.0;

        Self { data }
    }

    /// Constructs a new matrix from the given column vectors.
    #[inline]
    pub const fn from_cols(x_axis: Vector3, y_axis: Vector3, z_axis: Vector3) -> Self {
        let mut data: [f32; 9] = [0.0; 9];

        data[0] = x_axis.x;
        data[1] = x_axis.y;
        data[2] = x_axis.z;

        data[3] = y_axis.x;
        data[4] = y_axis.y;
        data[5] = y_axis.z;

        data[6] = z_axis.x;
        data[7] = z_axis.y;
        data[8] = z_axis.z;

        Self { data }
    }

    /// Creates a new rotation matrix.
    #[inline]
    pub fn create_rotation(rotation: Quaternion) -> Matrix3x3 {
        rotation.to_3x3()
    }

    /// Creates a new scale matrix.
    #[inline]
    pub fn create_scale(scale: Vector3) -> Matrix3x3 {
        let mut result = Matrix3x3::new();

        *result.mat_mut::<0, 0>() = scale.x;
        *result.mat_mut::<1, 1>() = scale.y;
        *result.mat_mut::<2, 2>() = scale.z;

        result
    }

    /// Access a single matrix value.
    /// `m[X][Y]`
    #[inline]
    pub fn mat<const X: usize, const Y: usize>(&self) -> f32 {
        self.data[X * 3 + Y]
    }

    /// Mutably access a single matrix value.
    /// `m[X][Y]`
    #[inline]
    pub fn mat_mut<const X: usize, const Y: usize>(&mut self) -> &mut f32 {
        &mut self.data[X * 3 + Y]
    }

    /// Returns the rotation component of this matrix.
    #[inline]
    pub fn rotation(&self) -> Quaternion {
        let (rotation, _) = self.decompose();

        rotation
    }

    /// Returns the scale component of this matrix.
    #[inline]
    pub fn scale(&self) -> Vector3 {
        let (_, scale) = self.decompose();

        scale
    }

    /// Decomposes the matrix into rotation, and scale components.
    #[inline]
    pub fn decompose(&self) -> (Quaternion, Vector3) {
        let mut x_axis = Vector3::new(self.mat::<0, 0>(), self.mat::<0, 1>(), self.mat::<0, 2>());
        let mut y_axis = Vector3::new(self.mat::<1, 0>(), self.mat::<1, 1>(), self.mat::<1, 2>());
        let mut z_axis = Vector3::new(self.mat::<2, 0>(), self.mat::<2, 1>(), self.mat::<2, 2>());

        let mut scale = Vector3::new(x_axis.length(), y_axis.length(), z_axis.length());

        if scale.x > f32::EPSILON {
            x_axis /= scale.x;
        }

        if scale.y > f32::EPSILON {
            y_axis /= scale.y;
        }

        if scale.z > f32::EPSILON {
            z_axis /= scale.z;
        }

        if x_axis.cross(y_axis).dot(z_axis) < 0.0 {
            scale.x = -scale.x;
            x_axis = -x_axis;
        }

        let trace = x_axis.x + y_axis.y + z_axis.z;
        let half: f32 = 0.5f32;

        let rotation = if trace >= 0.0f32 {
            let s = (1.0f32 + trace).sqrt();
            let w = half * s;
            let s = half / s;
            let x = (y_axis.z - z_axis.y) * s;
            let y = (z_axis.x - x_axis.z) * s;
            let z = (x_axis.y - y_axis.x) * s;

            Quaternion::new(x, y, z, w)
        } else if (x_axis.x > y_axis.y) && (x_axis.x > z_axis.z) {
            let s = ((x_axis.x - y_axis.y - z_axis.z) + 1.0f32).sqrt();
            let x = half * s;
            let s = half / s;
            let y = (y_axis.x + x_axis.y) * s;
            let z = (x_axis.z + z_axis.x) * s;
            let w = (y_axis.z - z_axis.y) * s;

            Quaternion::new(x, y, z, w)
        } else if y_axis.y > z_axis.z {
            let s = ((y_axis.y - x_axis.x - z_axis.z) + 1.0f32).sqrt();
            let y = half * s;
            let s = half / s;
            let z = (z_axis.y + y_axis.z) * s;
            let x = (y_axis.x + x_axis.y) * s;
            let w = (z_axis.x - x_axis.z) * s;

            Quaternion::new(x, y, z, w)
        } else {
            let s = ((z_axis.z - x_axis.x - y_axis.y) + 1.0f32).sqrt();
            let z = half * s;
            let s = half / s;
            let x = (x_axis.z + z_axis.x) * s;
            let y = (z_axis.y + y_axis.z) * s;
            let w = (x_axis.y - y_axis.x) * s;

            Quaternion::new(x, y, z, w)
        };

        (rotation, scale)
    }

    /// Returns the rotation of this matrix as euler angles.
    ///
    /// ### Note:
    /// This method assumes the matrix has no scale or skew.
    #[inline]
    pub fn to_euler(&self, measurment: Angles) -> Vector3 {
        let square_sum = (self.mat::<0, 0>() * self.mat::<0, 0>()
            + self.mat::<0, 1>() * self.mat::<0, 1>())
        .sqrt();

        let result = if square_sum > 0.00016 {
            Vector3::new(
                self.mat::<1, 2>().atan2(self.mat::<2, 2>()),
                (-self.mat::<0, 2>()).atan2(square_sum),
                self.mat::<0, 1>().atan2(self.mat::<0, 0>()),
            )
        } else {
            Vector3::new(
                (-self.mat::<2, 1>()).atan2(self.mat::<1, 1>()),
                (-self.mat::<0, 2>()).atan2(square_sum),
                0.0,
            )
        };

        if measurment == Angles::Degrees {
            Vector3::new(
                radians_to_degrees(result.x),
                radians_to_degrees(result.y),
                radians_to_degrees(result.z),
            )
        } else {
            result
        }
    }

    /// Reverses the byte order of the matrix.
    #[inline]
    #[unroll::unroll_for_loops]
    pub fn swap_bytes(self) -> Matrix3x3 {
        let mut result = Matrix3x3::new();

        for i in 0..9 {
            result.data[i] = f32::from_bits(self.data[i].to_bits().swap_bytes());
        }

        result
    }

    /// Swaps the handedness of this matrix.
    #[inline]
    pub fn swap_handedness(self) -> Self {
        let (rot, sca) = self.decompose();

        Self::create_rotation(Quaternion::new(-rot.z, rot.x, -rot.y, rot.w))
            * Self::create_scale(Vector3::new(sca.z, sca.x, sca.y))
    }

    /// Returns the transpose of this matrix.
    #[inline]
    #[unroll::unroll_for_loops]
    pub fn transpose(&self) -> Matrix3x3 {
        let mut result = Matrix3x3::new();

        for i in 0..3 {
            for j in 0..3 {
                *result.mat_mut::<i, j>() = self.mat::<j, i>();
            }
        }

        result
    }

    /// Converts this rotation matrix to a matrix.
    #[inline]
    pub fn to_4x4(self) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<0, 0>() = self.mat::<0, 0>();
        *result.mat_mut::<0, 1>() = self.mat::<0, 1>();
        *result.mat_mut::<0, 2>() = self.mat::<0, 2>();

        *result.mat_mut::<1, 0>() = self.mat::<1, 0>();
        *result.mat_mut::<1, 1>() = self.mat::<1, 1>();
        *result.mat_mut::<1, 2>() = self.mat::<1, 2>();

        *result.mat_mut::<2, 0>() = self.mat::<2, 0>();
        *result.mat_mut::<2, 1>() = self.mat::<2, 1>();
        *result.mat_mut::<2, 2>() = self.mat::<2, 2>();

        result
    }
}

impl cmp::PartialEq for Matrix3x3 {
    #[inline]
    #[unroll::unroll_for_loops]
    fn eq(&self, other: &Self) -> bool {
        for i in 0..9 {
            if (self.data[i] - other.data[i]).abs() >= f32::EPSILON {
                return false;
            }
        }

        true
    }
}

impl Default for Matrix3x3 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<[f32; 9]> for Matrix3x3 {
    fn from(value: [f32; 9]) -> Self {
        Self { data: value }
    }
}

impl fmt::Debug for Matrix3x3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix3x3")
            .field("m[0][0]", &self.mat::<0, 0>())
            .field("m[0][1]", &self.mat::<0, 1>())
            .field("m[0][2]", &self.mat::<0, 2>())
            .field("m[1][0]", &self.mat::<1, 0>())
            .field("m[1][1]", &self.mat::<1, 1>())
            .field("m[1][2]", &self.mat::<1, 2>())
            .field("m[2][0]", &self.mat::<2, 0>())
            .field("m[2][1]", &self.mat::<2, 1>())
            .field("m[2][2]", &self.mat::<2, 2>())
            .finish()
    }
}

impl ops::Add<Matrix3x3> for Matrix3x3 {
    type Output = Matrix3x3;

    #[inline]
    #[unroll::unroll_for_loops]
    fn add(self, rhs: Matrix3x3) -> Self::Output {
        let mut result = Matrix3x3::new();

        for i in 0..9 {
            result.data[i] = self.data[i] + rhs.data[i];
        }

        result
    }
}

impl ops::Sub<Matrix3x3> for Matrix3x3 {
    type Output = Matrix3x3;

    #[inline]
    #[unroll::unroll_for_loops]
    fn sub(self, rhs: Matrix3x3) -> Self::Output {
        let mut result = Matrix3x3::new();

        for i in 0..9 {
            result.data[i] = self.data[i] - rhs.data[i];
        }

        result
    }
}

impl ops::Mul<Matrix3x3> for Matrix3x3 {
    type Output = Matrix3x3;

    #[inline]
    #[unroll::unroll_for_loops]
    fn mul(self, rhs: Matrix3x3) -> Self::Output {
        let mut result = Matrix3x3::new();

        for i in 0..3 {
            for j in 0..3 {
                let mut value = 0.0;

                for k in 0..3 {
                    value += rhs.mat::<i, k>() * self.mat::<k, j>();
                }

                *result.mat_mut::<i, j>() = value;
            }
        }

        result
    }
}

impl ops::Div<f32> for Matrix3x3 {
    type Output = Matrix3x3;

    #[inline]
    #[unroll::unroll_for_loops]
    fn div(self, rhs: f32) -> Self::Output {
        let mut result = Matrix3x3::new();

        for i in 0..9 {
            result.data[i] = self.data[i] / rhs;
        }

        result
    }
}

impl ops::Index<usize> for Matrix3x3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
