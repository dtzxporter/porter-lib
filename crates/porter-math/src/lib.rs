#![deny(unsafe_code)]

mod angles;
mod matrix4x4;
mod quaternion;
mod rmatrix4x4;
mod vector2;
mod vector3;

pub use angles::*;
pub use matrix4x4::*;
pub use quaternion::*;
pub use rmatrix4x4::*;
pub use vector2::*;
pub use vector3::*;

pub use half::f16;

/// Converts degrees into radians.
pub fn degrees_to_radians(value: f32) -> f32 {
    (value * std::f32::consts::PI) / 180.0
}

/// Converts radians to degrees.
pub fn radians_to_degrees(value: f32) -> f32 {
    (value * 180.0) / std::f32::consts::PI
}

/// Normalizes a f32x4 array.
pub fn normalize_f32x4(mut array: [f32; 4]) -> [f32; 4] {
    let length_sq =
        array[0] * array[0] * array[1] * array[1] * array[2] * array[2] * array[3] * array[3];

    let length = length_sq.sqrt();

    array[0] /= length;
    array[1] /= length;
    array[2] /= length;
    array[3] /= length;

    array
}
