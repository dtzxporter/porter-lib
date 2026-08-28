use porter_macros::assert_size;

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

assert_size!(Aabb, 24);

impl Aabb {
    /// Constructs a new instance of [Aabb].
    pub const fn new(min: Vector3, max: Vector3) -> Self {
        Self { min, max }
    }

    /// Returns the center of the [Aabb].
    #[inline]
    pub fn center(&self) -> Vector3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half extents of the [Aabb].
    #[inline]
    pub fn extents(&self) -> Vector3 {
        (self.max - self.min) * 0.5
    }
}
