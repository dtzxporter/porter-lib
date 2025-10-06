use porter_math::PackedU8Vector4;

/// Represents the color of a vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VertexColor {
    /// The red channel in (0..=255).
    pub r: u8,
    /// The green channel in (0..=255).
    pub g: u8,
    /// The blue channel in (0..=255).
    pub b: u8,
    /// The alpha channel in (0..=255).
    pub a: u8,
}

impl VertexColor {
    /// Constructs a new vertex color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<VertexColor> for u32 {
    #[inline]
    fn from(value: VertexColor) -> Self {
        u32::from_le_bytes([value.r, value.g, value.b, value.a])
    }
}

impl From<PackedU8Vector4> for VertexColor {
    #[inline]
    fn from(value: PackedU8Vector4) -> Self {
        Self::new(value.x, value.y, value.z, value.w)
    }
}

impl From<[u8; 4]> for VertexColor {
    #[inline]
    fn from(value: [u8; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

impl Default for VertexColor {
    fn default() -> Self {
        Self::new(255, 255, 255, 255)
    }
}
