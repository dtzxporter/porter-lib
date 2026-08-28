use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector3;

/// Represents an joint used for an animation.
#[derive(Debug, Clone, Copy)]
pub struct Joint {
    pub parent: i32,
    pub reset_to_bind: bool,
    pub local_position: Vector3,
    pub local_rotation: Quaternion,
    pub local_scale: Vector3,
    pub world_position: Vector3,
    pub world_rotation: Quaternion,
    pub world_scale: Vector3,
}

impl Joint {
    /// Constructs a new instance of a joint.
    pub const fn new(parent: i32) -> Self {
        Self {
            parent,
            reset_to_bind: false,
            local_position: Vector3::zero(),
            local_rotation: Quaternion::identity(),
            local_scale: Vector3::one(),
            world_position: Vector3::zero(),
            world_rotation: Quaternion::identity(),
            world_scale: Vector3::one(),
        }
    }

    /// Sets whether or not to reset the joint to bind position before every frame.
    pub const fn reset_to_bind(mut self, enabled: bool) -> Self {
        self.reset_to_bind = enabled;
        self
    }

    /// Sets the local position.
    pub const fn local_position(mut self, position: Vector3) -> Self {
        self.local_position = position;
        self
    }

    /// Sets the local rotation.
    pub const fn local_rotation(mut self, rotation: Quaternion) -> Self {
        self.local_rotation = rotation;
        self
    }

    /// Sets the local scale.
    pub const fn local_scale(mut self, scale: Vector3) -> Self {
        self.local_scale = scale;
        self
    }

    /// Sets the world position.
    pub const fn world_position(mut self, position: Vector3) -> Self {
        self.world_position = position;
        self
    }

    /// Sets the world rotation.
    pub const fn world_rotation(mut self, rotation: Quaternion) -> Self {
        self.world_rotation = rotation;
        self
    }

    /// Sets the world scale.
    pub const fn world_scale(mut self, scale: Vector3) -> Self {
        self.world_scale = scale;
        self
    }

    /// Generates local transforms based on global transforms.
    pub fn generate_local_transforms(&mut self, parent: &Self) {
        let parent_rotation = parent.world_rotation.conjugate();

        let local_position =
            (self.world_position - parent.world_position).transform(&parent_rotation.to_4x4());
        let local_rotation = parent_rotation * self.world_rotation;

        let local_scale_mask = parent
            .world_scale
            .cmpne(Vector3::zero());

        let local_scale =
            (self.world_scale / parent.world_scale).select(local_scale_mask, Vector3::zero());

        self.local_position = local_position;
        self.local_rotation = local_rotation;
        self.local_scale = local_scale;
    }

    /// Generates world transforms based on local transforms.
    pub fn generate_world_transforms(&mut self, parent: &Self) {
        let parent_world = Matrix4x4::create_trs(
            parent.world_position,
            parent.world_rotation,
            parent.world_scale,
        );

        let local_position = Matrix4x4::create_position(self.local_position);

        let result = ((parent_world * local_position) * parent_world.inverse()).position();

        self.world_position = parent.world_position + result;
        self.world_rotation = parent.world_rotation * self.local_rotation;
        self.world_scale = parent.world_scale * self.local_scale;
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self::new(-1)
    }
}
