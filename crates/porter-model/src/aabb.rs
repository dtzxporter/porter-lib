use porter_math::Vector3;

/// Represents a 3D bounding box with min/max bounds.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    /// The minimum box bounds.
    pub min: Vector3,
    /// The maximum box bounds.
    pub max: Vector3,
}

impl Aabb {
    /// Constructs a new instance of [Aabb].
    pub const fn new(min: Vector3, max: Vector3) -> Self {
        Self { min, max }
    }
}
