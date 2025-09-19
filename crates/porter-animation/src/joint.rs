use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector3;

/// Represents a joint used for an animation.
#[derive(Debug, Clone, Copy)]
pub struct Joint {
    pub parent: i32,
    pub local_position: Vector3,
    pub local_rotation: Quaternion,
    pub world_position: Vector3,
    pub world_rotation: Quaternion,
}

impl Joint {
    /// Constructs a new instance of a joint.
    pub fn new(parent: i32) -> Self {
        Self {
            parent,
            local_position: Vector3::zero(),
            local_rotation: Quaternion::identity(),
            world_position: Vector3::zero(),
            world_rotation: Quaternion::identity(),
        }
    }

    /// Sets the local position.
    #[inline]
    pub fn local_position(mut self, position: Vector3) -> Self {
        self.local_position = position;
        self
    }

    /// Sets the local rotation.
    #[inline]
    pub fn local_rotation(mut self, rotation: Quaternion) -> Self {
        self.local_rotation = rotation;
        self
    }

    /// Sets the world position.
    #[inline]
    pub fn world_position(mut self, position: Vector3) -> Self {
        self.world_position = position;
        self
    }

    /// Sets the world rotation.
    #[inline]
    pub fn world_rotation(mut self, rotation: Quaternion) -> Self {
        self.world_rotation = rotation;
        self
    }

    /// Generates local transforms based on global transforms.
    pub fn generate_local_transforms(&mut self, parent: &Self) {
        let parent_rotation = parent.world_rotation.conjugate();

        let local_position =
            (self.world_position - parent.world_position).transform(&parent_rotation.to_4x4());
        let local_rotation = parent_rotation * self.world_rotation;

        self.local_position = local_position;
        self.local_rotation = local_rotation;
    }

    /// Generates world transforms based on local transforms.
    pub fn generate_world_transforms(&mut self, parent: &Self) {
        let parent_world = Matrix4x4::create_position(parent.world_position)
            * Matrix4x4::create_rotation(parent.world_rotation);

        let local_position = Matrix4x4::create_position(self.local_position);

        let result = ((parent_world * local_position) * parent_world.inverse()).position();

        self.world_position = parent.world_position + result;
        self.world_rotation = parent.world_rotation * self.local_rotation;
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self::new(-1)
    }
}
