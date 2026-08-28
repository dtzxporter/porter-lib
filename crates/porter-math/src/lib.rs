#![deny(unsafe_code)]

mod angles;
mod axis;
mod knot_vector;
mod matrix3x3;
mod matrix4x4;
mod quaternion;
mod quaternion2;
mod quaternion_spline;
mod rect;
mod rmatrix4x4;
mod vector2;
mod vector3;
mod vector3_spline;
mod vector4;

pub use angles::*;
pub use axis::*;
pub use knot_vector::*;
pub use matrix3x3::*;
pub use matrix4x4::*;
pub use quaternion::*;
pub use quaternion_spline::*;
pub use quaternion2::*;
pub use rect::*;
pub use rmatrix4x4::*;
pub use vector2::*;
pub use vector3::*;
pub use vector3_spline::*;
pub use vector4::*;

pub use half::f16;

/// Normalizes a f32 array.
pub fn normalize_array_f32<const SIZE: usize>(mut array: [f32; SIZE]) -> [f32; SIZE] {
    let mut sum: f32 = 0.0;

    #[allow(clippy::needless_range_loop)]
    for i in 0..SIZE {
        sum += array[i];
    }

    if sum > f32::EPSILON {
        #[allow(clippy::needless_range_loop)]
        for i in 0..SIZE {
            array[i] /= sum;
        }
    }

    array
}
