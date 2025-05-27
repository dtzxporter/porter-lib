use std::cmp;
use std::fmt;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Angles;
use crate::Matrix3x3;
use crate::Quaternion;
use crate::RMatrix4x4;
use crate::Vector3;
use crate::radians_to_degrees;

/// Represents a 4x4 matrix in column major order.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Matrix4x4 {
    data: [f32; 16],
}

assert_eq_size!([u8; 64], Matrix4x4);

impl Matrix4x4 {
    /// Constructs a new identity matrix.
    #[inline]
    pub const fn new() -> Self {
        let mut data: [f32; 16] = [0.0; 16];

        data[0] = 1.0;
        data[1] = 0.0;
        data[2] = 0.0;
        data[3] = 0.0;
        data[4] = 0.0;
        data[5] = 1.0;
        data[6] = 0.0;
        data[7] = 0.0;
        data[8] = 0.0;
        data[9] = 0.0;
        data[10] = 1.0;
        data[11] = 0.0;
        data[12] = 0.0;
        data[13] = 0.0;
        data[14] = 0.0;
        data[15] = 1.0;

        Self { data }
    }

    /// Constructs a new perspective fov matrix.
    #[inline]
    pub fn perspective_fov(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let mut result = Matrix4x4::new();

        let top = near * (0.5 * crate::degrees_to_radians(fov)).tan();
        let bottom = -top;
        let right = top * aspect;
        let left = -right;

        let x = 2.0 * near / (right - left);
        let y = 2.0 * near / (top - bottom);
        let a = (right + left) / (right - left);
        let b = (top + bottom) / (top - bottom);
        let c = -(far + near) / (far - near);
        let d = -(2.0 * far * near) / (far - near);

        *result.mat_mut::<0, 0>() = x;
        *result.mat_mut::<0, 1>() = 0.0;
        *result.mat_mut::<0, 2>() = 0.0;
        *result.mat_mut::<0, 3>() = 0.0;

        *result.mat_mut::<1, 0>() = 0.0;
        *result.mat_mut::<1, 1>() = y;
        *result.mat_mut::<1, 2>() = 0.0;
        *result.mat_mut::<1, 3>() = 0.0;

        *result.mat_mut::<2, 0>() = a;
        *result.mat_mut::<2, 1>() = b;
        *result.mat_mut::<2, 2>() = c;
        *result.mat_mut::<2, 3>() = -1.0;

        *result.mat_mut::<3, 0>() = 0.0;
        *result.mat_mut::<3, 1>() = 0.0;
        *result.mat_mut::<3, 2>() = d;
        *result.mat_mut::<3, 3>() = 0.0;

        result
    }

    /// Constructs a new look at matrix.
    #[inline]
    pub fn look_at(from: Vector3, to: Vector3, right: Vector3) -> Self {
        let mut result = Matrix4x4::new();

        let z = (from - to).normalized();
        let x = right.cross(z).normalized();
        let y = z.cross(x);

        *result.mat_mut::<0, 0>() = x.x;
        *result.mat_mut::<0, 1>() = y.x;
        *result.mat_mut::<0, 2>() = z.x;
        *result.mat_mut::<0, 3>() = 0.0;

        *result.mat_mut::<1, 0>() = x.y;
        *result.mat_mut::<1, 1>() = y.y;
        *result.mat_mut::<1, 2>() = z.y;
        *result.mat_mut::<1, 3>() = 0.0;

        *result.mat_mut::<2, 0>() = x.z;
        *result.mat_mut::<2, 1>() = y.z;
        *result.mat_mut::<2, 2>() = z.z;
        *result.mat_mut::<2, 3>() = 0.0;

        *result.mat_mut::<3, 0>() = -((x.x * from.x) + (x.y * from.y) + (x.z * from.z));
        *result.mat_mut::<3, 1>() = -((y.x * from.x) + (y.y * from.y) + (y.z * from.z));
        *result.mat_mut::<3, 2>() = -((z.x * from.x) + (z.y * from.y) + (z.z * from.z));
        *result.mat_mut::<3, 3>() = 1.0;

        result
    }

    /// Constructs a new orthographic matrix.
    #[inline]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<0, 0>() = 2.0 / (right - left);
        *result.mat_mut::<0, 1>() = 0.0;
        *result.mat_mut::<0, 2>() = 0.0;
        *result.mat_mut::<0, 3>() = 0.0;

        *result.mat_mut::<1, 0>() = 0.0;
        *result.mat_mut::<1, 1>() = 2.0 / (top - bottom);
        *result.mat_mut::<1, 2>() = 0.0;
        *result.mat_mut::<1, 3>() = 0.0;

        *result.mat_mut::<2, 0>() = 0.0;
        *result.mat_mut::<2, 1>() = 0.0;
        *result.mat_mut::<2, 2>() = 1.0 / (near - far);
        *result.mat_mut::<2, 3>() = 0.0;

        *result.mat_mut::<3, 0>() = (left + right) / (left - right);
        *result.mat_mut::<3, 1>() = (top + bottom) / (bottom - top);
        *result.mat_mut::<3, 2>() = near / (near - far);
        *result.mat_mut::<3, 3>() = 1.0;

        result
    }

    /// Creates a new position matrix.
    #[inline]
    pub fn create_position(position: Vector3) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<3, 0>() = position.x;
        *result.mat_mut::<3, 1>() = position.y;
        *result.mat_mut::<3, 2>() = position.z;

        result
    }

    /// Creates a new rotation matrix.
    #[inline]
    pub fn create_rotation(rotation: Quaternion) -> Matrix4x4 {
        rotation.to_4x4()
    }

    /// Creates a new scale matrix.
    #[inline]
    pub fn create_scale(scale: Vector3) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<0, 0>() = scale.x;
        *result.mat_mut::<1, 1>() = scale.y;
        *result.mat_mut::<2, 2>() = scale.z;

        result
    }

    /// Access a single matrix value.
    /// `m[X][Y]`
    #[inline]
    pub fn mat<const X: usize, const Y: usize>(&self) -> f32 {
        self.data[X * 4 + Y]
    }

    /// Mutably access a single matrix value.
    /// `m[X][Y]`
    #[inline]
    pub fn mat_mut<const X: usize, const Y: usize>(&mut self) -> &mut f32 {
        &mut self.data[X * 4 + Y]
    }

    /// Returns the position component of this matrix.
    #[inline]
    pub fn position(&self) -> Vector3 {
        let (position, _, _) = self.decompose();

        position
    }

    /// Returns the rotation component of this matrix.
    #[inline]
    pub fn rotation(&self) -> Quaternion {
        let (_, rotation, _) = self.decompose();

        rotation
    }

    /// Returns the scale component of this matrix.
    #[inline]
    pub fn scale(&self) -> Vector3 {
        let (_, _, scale) = self.decompose();

        scale
    }

    /// Decomposes the matrix into position, rotation, and scale components.
    #[inline]
    pub fn decompose(&self) -> (Vector3, Quaternion, Vector3) {
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

        let position = Vector3::new(self.mat::<3, 0>(), self.mat::<3, 1>(), self.mat::<3, 2>());

        (position, rotation, scale)
    }

    /// Returns the rotation of this matrix as euler angles.
    /// ### Note:
    /// This method assumes the matrix has no scale or skew.
    #[inline]
    pub fn to_euler(&self, angles: Angles) -> Vector3 {
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

        if angles == Angles::Degrees {
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
    pub fn swap_bytes(self) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        for i in 0..16 {
            result.data[i] = f32::from_bits(self.data[i].to_bits().swap_bytes());
        }

        result
    }

    /// Swaps the handedness of this matrix.
    #[inline]
    pub fn swap_handedness(self) -> Self {
        let (pos, rot, sca) = self.decompose();

        Self::create_position(Vector3::new(pos.z, -pos.x, pos.y))
            * Self::create_rotation(Quaternion::new(-rot.z, rot.x, -rot.y, rot.w))
            * Self::create_scale(Vector3::new(sca.z, sca.x, sca.y))
    }

    /// Returns the transpose of this matrix.
    #[inline]
    #[unroll::unroll_for_loops]
    pub fn transpose(&self) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                *result.mat_mut::<i, j>() = self.mat::<j, i>();
            }
        }

        result
    }

    /// Calculates the matrix determinant.
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.mat::<3, 0>() * self.mat::<2, 1>() * self.mat::<1, 2>() * self.mat::<0, 3>()
            - self.mat::<2, 0>() * self.mat::<3, 1>() * self.mat::<1, 2>() * self.mat::<0, 3>()
            - self.mat::<3, 0>() * self.mat::<1, 1>() * self.mat::<2, 2>() * self.mat::<0, 3>()
            + self.mat::<1, 0>() * self.mat::<3, 1>() * self.mat::<2, 2>() * self.mat::<0, 3>()
            + self.mat::<2, 0>() * self.mat::<1, 1>() * self.mat::<3, 2>() * self.mat::<0, 3>()
            - self.mat::<1, 0>() * self.mat::<2, 1>() * self.mat::<3, 2>() * self.mat::<0, 3>()
            - self.mat::<3, 0>() * self.mat::<2, 1>() * self.mat::<0, 2>() * self.mat::<1, 3>()
            + self.mat::<2, 0>() * self.mat::<3, 1>() * self.mat::<0, 2>() * self.mat::<1, 3>()
            + self.mat::<3, 0>() * self.mat::<0, 1>() * self.mat::<2, 2>() * self.mat::<1, 3>()
            - self.mat::<0, 0>() * self.mat::<3, 1>() * self.mat::<2, 2>() * self.mat::<1, 3>()
            - self.mat::<2, 0>() * self.mat::<0, 1>() * self.mat::<3, 2>() * self.mat::<1, 3>()
            + self.mat::<0, 0>() * self.mat::<2, 1>() * self.mat::<3, 2>() * self.mat::<1, 3>()
            + self.mat::<3, 0>() * self.mat::<1, 1>() * self.mat::<0, 2>() * self.mat::<2, 3>()
            - self.mat::<1, 0>() * self.mat::<3, 1>() * self.mat::<0, 2>() * self.mat::<2, 3>()
            - self.mat::<3, 0>() * self.mat::<0, 1>() * self.mat::<1, 2>() * self.mat::<2, 3>()
            + self.mat::<0, 0>() * self.mat::<3, 1>() * self.mat::<1, 2>() * self.mat::<2, 3>()
            + self.mat::<1, 0>() * self.mat::<0, 1>() * self.mat::<3, 2>() * self.mat::<2, 3>()
            - self.mat::<0, 0>() * self.mat::<1, 1>() * self.mat::<3, 2>() * self.mat::<2, 3>()
            - self.mat::<2, 0>() * self.mat::<1, 1>() * self.mat::<0, 2>() * self.mat::<3, 3>()
            + self.mat::<1, 0>() * self.mat::<2, 1>() * self.mat::<0, 2>() * self.mat::<3, 3>()
            + self.mat::<2, 0>() * self.mat::<0, 1>() * self.mat::<1, 2>() * self.mat::<3, 3>()
            - self.mat::<0, 0>() * self.mat::<2, 1>() * self.mat::<1, 2>() * self.mat::<3, 3>()
            - self.mat::<1, 0>() * self.mat::<0, 1>() * self.mat::<2, 2>() * self.mat::<3, 3>()
            + self.mat::<0, 0>() * self.mat::<1, 1>() * self.mat::<2, 2>() * self.mat::<3, 3>()
    }

    /// Calculates the inverse of this matrix.
    #[inline]
    pub fn inverse(&self) -> Self {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<0, 0>() = self.mat::<2, 1>() * self.mat::<3, 2>() * self.mat::<1, 3>()
            - self.mat::<3, 1>() * self.mat::<2, 2>() * self.mat::<1, 3>()
            + self.mat::<3, 1>() * self.mat::<1, 2>() * self.mat::<2, 3>()
            - self.mat::<1, 1>() * self.mat::<3, 2>() * self.mat::<2, 3>()
            - self.mat::<2, 1>() * self.mat::<1, 2>() * self.mat::<3, 3>()
            + self.mat::<1, 1>() * self.mat::<2, 2>() * self.mat::<3, 3>();

        *result.mat_mut::<1, 0>() = self.mat::<3, 0>() * self.mat::<2, 2>() * self.mat::<1, 3>()
            - self.mat::<2, 0>() * self.mat::<3, 2>() * self.mat::<1, 3>()
            - self.mat::<3, 0>() * self.mat::<1, 2>() * self.mat::<2, 3>()
            + self.mat::<1, 0>() * self.mat::<3, 2>() * self.mat::<2, 3>()
            + self.mat::<2, 0>() * self.mat::<1, 2>() * self.mat::<3, 3>()
            - self.mat::<1, 0>() * self.mat::<2, 2>() * self.mat::<3, 3>();

        *result.mat_mut::<2, 0>() = self.mat::<2, 0>() * self.mat::<3, 1>() * self.mat::<1, 3>()
            - self.mat::<3, 0>() * self.mat::<2, 1>() * self.mat::<1, 3>()
            + self.mat::<3, 0>() * self.mat::<1, 1>() * self.mat::<2, 3>()
            - self.mat::<1, 0>() * self.mat::<3, 1>() * self.mat::<2, 3>()
            - self.mat::<2, 0>() * self.mat::<1, 1>() * self.mat::<3, 3>()
            + self.mat::<1, 0>() * self.mat::<2, 1>() * self.mat::<3, 3>();

        *result.mat_mut::<3, 0>() = self.mat::<3, 0>() * self.mat::<2, 1>() * self.mat::<1, 2>()
            - self.mat::<2, 0>() * self.mat::<3, 1>() * self.mat::<1, 2>()
            - self.mat::<3, 0>() * self.mat::<1, 1>() * self.mat::<2, 2>()
            + self.mat::<1, 0>() * self.mat::<3, 1>() * self.mat::<2, 2>()
            + self.mat::<2, 0>() * self.mat::<1, 1>() * self.mat::<3, 2>()
            - self.mat::<1, 0>() * self.mat::<2, 1>() * self.mat::<3, 2>();

        *result.mat_mut::<0, 1>() = self.mat::<3, 1>() * self.mat::<2, 2>() * self.mat::<0, 3>()
            - self.mat::<2, 1>() * self.mat::<3, 2>() * self.mat::<0, 3>()
            - self.mat::<3, 1>() * self.mat::<0, 2>() * self.mat::<2, 3>()
            + self.mat::<0, 1>() * self.mat::<3, 2>() * self.mat::<2, 3>()
            + self.mat::<2, 1>() * self.mat::<0, 2>() * self.mat::<3, 3>()
            - self.mat::<0, 1>() * self.mat::<2, 2>() * self.mat::<3, 3>();

        *result.mat_mut::<1, 1>() = self.mat::<2, 0>() * self.mat::<3, 2>() * self.mat::<0, 3>()
            - self.mat::<3, 0>() * self.mat::<2, 2>() * self.mat::<0, 3>()
            + self.mat::<3, 0>() * self.mat::<0, 2>() * self.mat::<2, 3>()
            - self.mat::<0, 0>() * self.mat::<3, 2>() * self.mat::<2, 3>()
            - self.mat::<2, 0>() * self.mat::<0, 2>() * self.mat::<3, 3>()
            + self.mat::<0, 0>() * self.mat::<2, 2>() * self.mat::<3, 3>();

        *result.mat_mut::<2, 1>() = self.mat::<3, 0>() * self.mat::<2, 1>() * self.mat::<0, 3>()
            - self.mat::<2, 0>() * self.mat::<3, 1>() * self.mat::<0, 3>()
            - self.mat::<3, 0>() * self.mat::<0, 1>() * self.mat::<2, 3>()
            + self.mat::<0, 0>() * self.mat::<3, 1>() * self.mat::<2, 3>()
            + self.mat::<2, 0>() * self.mat::<0, 1>() * self.mat::<3, 3>()
            - self.mat::<0, 0>() * self.mat::<2, 1>() * self.mat::<3, 3>();

        *result.mat_mut::<3, 1>() = self.mat::<2, 0>() * self.mat::<3, 1>() * self.mat::<0, 2>()
            - self.mat::<3, 0>() * self.mat::<2, 1>() * self.mat::<0, 2>()
            + self.mat::<3, 0>() * self.mat::<0, 1>() * self.mat::<2, 2>()
            - self.mat::<0, 0>() * self.mat::<3, 1>() * self.mat::<2, 2>()
            - self.mat::<2, 0>() * self.mat::<0, 1>() * self.mat::<3, 2>()
            + self.mat::<0, 0>() * self.mat::<2, 1>() * self.mat::<3, 2>();

        *result.mat_mut::<0, 2>() = self.mat::<1, 1>() * self.mat::<3, 2>() * self.mat::<0, 3>()
            - self.mat::<3, 1>() * self.mat::<1, 2>() * self.mat::<0, 3>()
            + self.mat::<3, 1>() * self.mat::<0, 2>() * self.mat::<1, 3>()
            - self.mat::<0, 1>() * self.mat::<3, 2>() * self.mat::<1, 3>()
            - self.mat::<1, 1>() * self.mat::<0, 2>() * self.mat::<3, 3>()
            + self.mat::<0, 1>() * self.mat::<1, 2>() * self.mat::<3, 3>();

        *result.mat_mut::<1, 2>() = self.mat::<3, 0>() * self.mat::<1, 2>() * self.mat::<0, 3>()
            - self.mat::<1, 0>() * self.mat::<3, 2>() * self.mat::<0, 3>()
            - self.mat::<3, 0>() * self.mat::<0, 2>() * self.mat::<1, 3>()
            + self.mat::<0, 0>() * self.mat::<3, 2>() * self.mat::<1, 3>()
            + self.mat::<1, 0>() * self.mat::<0, 2>() * self.mat::<3, 3>()
            - self.mat::<0, 0>() * self.mat::<1, 2>() * self.mat::<3, 3>();

        *result.mat_mut::<2, 2>() = self.mat::<1, 0>() * self.mat::<3, 1>() * self.mat::<0, 3>()
            - self.mat::<3, 0>() * self.mat::<1, 1>() * self.mat::<0, 3>()
            + self.mat::<3, 0>() * self.mat::<0, 1>() * self.mat::<1, 3>()
            - self.mat::<0, 0>() * self.mat::<3, 1>() * self.mat::<1, 3>()
            - self.mat::<1, 0>() * self.mat::<0, 1>() * self.mat::<3, 3>()
            + self.mat::<0, 0>() * self.mat::<1, 1>() * self.mat::<3, 3>();

        *result.mat_mut::<3, 2>() = self.mat::<3, 0>() * self.mat::<1, 1>() * self.mat::<0, 2>()
            - self.mat::<1, 0>() * self.mat::<3, 1>() * self.mat::<0, 2>()
            - self.mat::<3, 0>() * self.mat::<0, 1>() * self.mat::<1, 2>()
            + self.mat::<0, 0>() * self.mat::<3, 1>() * self.mat::<1, 2>()
            + self.mat::<1, 0>() * self.mat::<0, 1>() * self.mat::<3, 2>()
            - self.mat::<0, 0>() * self.mat::<1, 1>() * self.mat::<3, 2>();

        *result.mat_mut::<0, 3>() = self.mat::<2, 1>() * self.mat::<1, 2>() * self.mat::<0, 3>()
            - self.mat::<1, 1>() * self.mat::<2, 2>() * self.mat::<0, 3>()
            - self.mat::<2, 1>() * self.mat::<0, 2>() * self.mat::<1, 3>()
            + self.mat::<0, 1>() * self.mat::<2, 2>() * self.mat::<1, 3>()
            + self.mat::<1, 1>() * self.mat::<0, 2>() * self.mat::<2, 3>()
            - self.mat::<0, 1>() * self.mat::<1, 2>() * self.mat::<2, 3>();

        *result.mat_mut::<1, 3>() = self.mat::<1, 0>() * self.mat::<2, 2>() * self.mat::<0, 3>()
            - self.mat::<2, 0>() * self.mat::<1, 2>() * self.mat::<0, 3>()
            + self.mat::<2, 0>() * self.mat::<0, 2>() * self.mat::<1, 3>()
            - self.mat::<0, 0>() * self.mat::<2, 2>() * self.mat::<1, 3>()
            - self.mat::<1, 0>() * self.mat::<0, 2>() * self.mat::<2, 3>()
            + self.mat::<0, 0>() * self.mat::<1, 2>() * self.mat::<2, 3>();

        *result.mat_mut::<2, 3>() = self.mat::<2, 0>() * self.mat::<1, 1>() * self.mat::<0, 3>()
            - self.mat::<1, 0>() * self.mat::<2, 1>() * self.mat::<0, 3>()
            - self.mat::<2, 0>() * self.mat::<0, 1>() * self.mat::<1, 3>()
            + self.mat::<0, 0>() * self.mat::<2, 1>() * self.mat::<1, 3>()
            + self.mat::<1, 0>() * self.mat::<0, 1>() * self.mat::<2, 3>()
            - self.mat::<0, 0>() * self.mat::<1, 1>() * self.mat::<2, 3>();

        *result.mat_mut::<3, 3>() = self.mat::<1, 0>() * self.mat::<2, 1>() * self.mat::<0, 2>()
            - self.mat::<2, 0>() * self.mat::<1, 1>() * self.mat::<0, 2>()
            + self.mat::<2, 0>() * self.mat::<0, 1>() * self.mat::<1, 2>()
            - self.mat::<0, 0>() * self.mat::<2, 1>() * self.mat::<1, 2>()
            - self.mat::<1, 0>() * self.mat::<0, 1>() * self.mat::<2, 2>()
            + self.mat::<0, 0>() * self.mat::<1, 1>() * self.mat::<2, 2>();

        result / self.determinant()
    }

    /// Converts this matrix to a rotation matrix.
    #[inline]
    pub fn to_3x3(self) -> Matrix3x3 {
        let mut result = Matrix3x3::new();

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

    /// Converts this column major matrix to row matrix order.
    #[inline]
    #[unroll::unroll_for_loops]
    pub fn to_row_major(&self) -> RMatrix4x4 {
        let mut result = RMatrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                *result.mat_mut::<i, j>() = self.mat::<j, i>();
            }
        }

        result
    }
}

impl cmp::PartialEq for Matrix4x4 {
    #[inline]
    #[unroll::unroll_for_loops]
    fn eq(&self, other: &Self) -> bool {
        for i in 0..16 {
            if (self.data[i] - other.data[i]).abs() >= f32::EPSILON {
                return false;
            }
        }

        true
    }
}

impl Default for Matrix4x4 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl From<[f32; 16]> for Matrix4x4 {
    fn from(value: [f32; 16]) -> Self {
        Self { data: value }
    }
}

impl fmt::Debug for Matrix4x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix4x4")
            .field("m[0][0]", &self.mat::<0, 0>())
            .field("m[0][1]", &self.mat::<0, 1>())
            .field("m[0][2]", &self.mat::<0, 2>())
            .field("m[0][3]", &self.mat::<0, 3>())
            .field("m[1][0]", &self.mat::<1, 0>())
            .field("m[1][1]", &self.mat::<1, 1>())
            .field("m[1][2]", &self.mat::<1, 2>())
            .field("m[1][3]", &self.mat::<1, 3>())
            .field("m[2][0]", &self.mat::<2, 0>())
            .field("m[2][1]", &self.mat::<2, 1>())
            .field("m[2][2]", &self.mat::<2, 2>())
            .field("m[2][3]", &self.mat::<2, 3>())
            .field("m[3][0]", &self.mat::<3, 0>())
            .field("m[3][1]", &self.mat::<3, 1>())
            .field("m[3][2]", &self.mat::<3, 2>())
            .field("m[3][3]", &self.mat::<3, 3>())
            .finish()
    }
}

impl ops::Add<Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;

    #[inline]
    #[unroll::unroll_for_loops]
    fn add(self, rhs: Matrix4x4) -> Self::Output {
        let mut result = Matrix4x4::new();

        for i in 0..16 {
            result.data[i] = self.data[i] + rhs.data[i];
        }

        result
    }
}

impl ops::Sub<Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;

    #[inline]
    #[unroll::unroll_for_loops]
    fn sub(self, rhs: Matrix4x4) -> Self::Output {
        let mut result = Matrix4x4::new();

        for i in 0..16 {
            result.data[i] = self.data[i] - rhs.data[i];
        }

        result
    }
}

impl ops::Mul<Matrix4x4> for Matrix4x4 {
    type Output = Matrix4x4;

    #[inline]
    #[unroll::unroll_for_loops]
    fn mul(self, rhs: Matrix4x4) -> Self::Output {
        let mut result = Matrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                let mut value = 0.0;

                for k in 0..4 {
                    value += rhs.mat::<i, k>() * self.mat::<k, j>();
                }

                *result.mat_mut::<i, j>() = value;
            }
        }

        result
    }
}

impl ops::Div<f32> for Matrix4x4 {
    type Output = Matrix4x4;

    #[inline]
    #[unroll::unroll_for_loops]
    fn div(self, rhs: f32) -> Self::Output {
        let mut result = Matrix4x4::new();

        for i in 0..16 {
            result.data[i] = self.data[i] / rhs;
        }

        result
    }
}

impl ops::Index<usize> for Matrix4x4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
