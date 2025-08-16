/// Returns the base 2 logarithm of the number.
#[inline]
fn log2(x: f32) -> f32 {
    let vx = x.to_bits();
    let mx = f32::from_bits((vx & 0x007FFFFF_u32) | 0x3f000000);

    let mut y = vx as f32;

    y *= 1.1920929e-7f32;
    y - 124.22552f32 - 1.4980303f32 * mx - 1.72588f32 / (0.35208872f32 + mx)
}

/// Raises 2 to a floating point power.
#[inline]
fn pow2(p: f32) -> f32 {
    let offset = if p < 0.0 { 1.0_f32 } else { 0.0_f32 };
    let clipp = if p < -126.0 { -126.0_f32 } else { p };

    let w = clipp as i32;
    let z = clipp - (w as f32) + offset;
    let v = ((1 << 23) as f32
        * (clipp + 121.274055f32 + 27.728024f32 / (4.8425255f32 - z) - 1.4901291f32 * z))
        as u32;

    f32::from_bits(v)
}

/// Raises a number to a floating point power.
#[inline]
fn powf(x: f32, p: f32) -> f32 {
    pow2(p * log2(x))
}

/// Converts a linear color value to an srgb color value.
#[inline(always)]
pub fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.0031308 {
        value * 12.92
    } else {
        (powf(value, 1.0 / 2.4) * 1.055) + -0.055
    }
}

/// Converts a srgb color to a linear color value.
#[inline(always)]
pub fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        powf((value + 0.055) / 1.055, 2.4)
    }
}
