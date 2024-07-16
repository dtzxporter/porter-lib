use std::fmt;
use std::ops;

use static_assertions::assert_eq_size;

use crate::Matrix4x4;

/// Represents a 4x4 matrix in row major order.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct RMatrix4x4 {
    data: [f32; 16],
}

assert_eq_size!([u8; 64], RMatrix4x4);

impl RMatrix4x4 {
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
    pub fn mat<const X: usize, const Y: usize>(&self) -> f32 {
        self.data[X * 4 + Y]
    }

    #[inline]
    pub fn mat_mut<const X: usize, const Y: usize>(&mut self) -> &mut f32 {
        &mut self.data[X * 4 + Y]
    }

    #[inline]
    #[unroll::unroll_for_loops]
    pub fn column_major(&self) -> Matrix4x4 {
        let mut result = Matrix4x4::new();

        for i in 0..4 {
            for j in 0..4 {
                *result.mat_mut::<i, j>() = self.mat::<j, i>();
            }
        }

        result
    }

    #[inline]
    #[unroll::unroll_for_loops]
    pub fn swap_bytes(&self) -> RMatrix4x4 {
        let mut result = RMatrix4x4::new();

        for i in 0..16 {
            result.data[i] = f32::from_bits(self.data[i].to_bits().swap_bytes());
        }

        result
    }
}

impl Default for RMatrix4x4 {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Matrix4x4> for RMatrix4x4 {
    fn from(value: Matrix4x4) -> Self {
        value.row_major()
    }
}

impl fmt::Debug for RMatrix4x4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RMatrix4x4")
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

impl ops::Index<usize> for RMatrix4x4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
