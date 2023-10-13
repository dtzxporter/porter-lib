/// Represents the color of a vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VertexColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl VertexColor {
    /// Constructs a new vertex color.
    #[inline]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<VertexColor> for u32 {
    fn from(value: VertexColor) -> Self {
        u32::from_le_bytes([value.r, value.g, value.b, value.a])
    }
}
