use static_assertions::assert_eq_size;

/// Represents a weight influence for a vertex.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct VertexWeight {
    pub bone: u16,
    pub value: f32,
}

assert_eq_size!([u8; 0x6], VertexWeight);

impl VertexWeight {
    /// Constructs a new vertex weight.
    #[inline]
    pub const fn new(bone: u16, value: f32) -> Self {
        Self { bone, value }
    }
}
