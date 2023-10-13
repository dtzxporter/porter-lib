use std::cmp;
use std::fmt;
use std::ops;

use static_assertions::assert_eq_size;

use crate::radians_to_degrees;
use crate::Angles;
use crate::Quaternion;
use crate::RMatrix4x4;
use crate::Vector3;

/// Represents a 4x4 matrix in column major order.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct Matrix4x4 {
    data: [f32; 16],
}

assert_eq_size!([u8; 64], Matrix4x4);

impl Matrix4x4 {
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

    #[inline]
    pub fn create_position(position: Vector3) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<3, 0>() = position.x;
        *result.mat_mut::<3, 1>() = position.y;
        *result.mat_mut::<3, 2>() = position.z;

        result
    }

    #[inline]
    pub fn create_scale(scale: Vector3) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        *result.mat_mut::<0, 0>() = scale.x;
        *result.mat_mut::<1, 1>() = scale.y;
        *result.mat_mut::<2, 2>() = scale.z;

        result
    }

    #[inline]
    pub fn mat<const X: usize, const Y: usize>(&self) -> f32 {
        self.data[X * 4 + Y]
    }

    #[inline]
    pub fn mat_mut<const X: usize, const Y: usize>(&mut self) -> &mut f32 {
        &mut self.data[X * 4 + Y]
    }

    #[inline]
    pub fn position(&self) -> Vector3 {
        Vector3::new(self.mat::<3, 0>(), self.mat::<3, 1>(), self.mat::<3, 2>())
    }

    #[inline]
    pub fn rotation(&self) -> Quaternion {
        if (self.mat::<0, 0>() + self.mat::<1, 1>() + self.mat::<2, 2>()) > 0.0 {
            let scale =
                (self.mat::<0, 0>() + self.mat::<1, 1>() + self.mat::<2, 2>() + 1.0).sqrt() * 2.0;

            Quaternion::new(
                (self.mat::<2, 1>() - self.mat::<1, 2>()) / scale,
                (self.mat::<0, 2>() - self.mat::<2, 0>()) / scale,
                (self.mat::<1, 0>() - self.mat::<0, 1>()) / scale,
                0.25 * scale,
            )
        } else if (self.mat::<0, 0>() > self.mat::<1, 1>())
            && (self.mat::<0, 0>() > self.mat::<2, 2>())
        {
            let scale =
                (1.0 + self.mat::<0, 0>() - self.mat::<1, 1>() - self.mat::<2, 2>()).sqrt() * 2.0;

            Quaternion::new(
                0.25 * scale,
                (self.mat::<0, 1>() + self.mat::<1, 0>()) / scale,
                (self.mat::<0, 2>() + self.mat::<2, 0>()) / scale,
                (self.mat::<2, 1>() - self.mat::<1, 2>()) / scale,
            )
        } else if self.mat::<1, 1>() > self.mat::<2, 2>() {
            let scale =
                (1.0 + self.mat::<1, 1>() - self.mat::<0, 0>() - self.mat::<2, 2>()).sqrt() * 2.0;

            Quaternion::new(
                (self.mat::<0, 1>() + self.mat::<1, 0>()) / scale,
                0.25 * scale,
                (self.mat::<1, 2>() + self.mat::<2, 1>()) / scale,
                (self.mat::<0, 2>() - self.mat::<2, 0>()) / scale,
            )
        } else {
            let scale = (1.0 + self.mat::<2, 2>() - self.mat::<0, 0>() - self.mat::<1, 1>()) * 2.0;

            Quaternion::new(
                (self.mat::<0, 2>() + self.mat::<2, 0>()) / scale,
                (self.mat::<1, 2>() + self.mat::<2, 1>()) / scale,
                0.25 * scale,
                (self.mat::<1, 0>() - self.mat::<0, 1>()) / scale,
            )
        }
    }

    #[inline]
    pub fn scale(&self) -> Vector3 {
        let x = Vector3::new(self.mat::<0, 0>(), self.mat::<0, 1>(), self.mat::<0, 2>());
        let y = Vector3::new(self.mat::<1, 0>(), self.mat::<1, 1>(), self.mat::<1, 2>());
        let z = Vector3::new(self.mat::<2, 0>(), self.mat::<2, 1>(), self.mat::<2, 2>());

        Vector3::new(x.length(), y.length(), z.length())
    }

    #[inline]
    pub fn euler_angles(&self, measurment: Angles) -> Vector3 {
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

    #[inline]
    #[unroll::unroll_for_loops]
    pub fn row_major(&self) -> RMatrix4x4 {
        let mut result = RMatrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                *result.mat_mut::<i, j>() = self.mat::<j, i>();
            }
        }

        result
    }

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
