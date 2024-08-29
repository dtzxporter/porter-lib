use static_assertions::assert_eq_size;

/// The type of a bone id.
pub type WeightBoneId = u16;

/// Represents a weight influence for a vertex.
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct VertexWeight {
    /// The index of the bone which influences this vertex.
    pub bone: WeightBoneId,
    /// The percent in (0.0..=1.0) of influence this bone has on the vertex.
    pub value: f32,
}

assert_eq_size!([u8; 0x6], VertexWeight);

impl VertexWeight {
    /// Constructs a new vertex weight.
    pub const fn new(bone: WeightBoneId, value: f32) -> Self {
        Self { bone, value }
    }
}
