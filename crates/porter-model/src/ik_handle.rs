/// A 3d bone inverse kinematics handle.
#[derive(Debug, Clone)]
pub struct IKHandle {
    pub name: Option<String>,
    pub start_bone: usize,
    pub end_bone: usize,
    pub target_bone: Option<usize>,
    pub pole_vector_bone: Option<usize>,
    pub pole_bone: Option<usize>,
    pub use_target_rotation: bool,
}

impl IKHandle {
    /// Constructs a new instance of an ik handle.
    pub fn new(name: Option<String>, start_bone: usize, end_bone: usize) -> Self {
        Self {
            name,
            start_bone,
            end_bone,
            target_bone: None,
            pole_vector_bone: None,
            pole_bone: None,
            use_target_rotation: false,
        }
    }

    /// Sets the target bone index.
    #[inline]
    pub fn target_bone(mut self, bone: usize) -> Self {
        self.target_bone = Some(bone);
        self
    }

    /// Sets the pole vector bone index.
    #[inline]
    pub fn pole_vector_bone(mut self, bone: usize) -> Self {
        self.pole_vector_bone = Some(bone);
        self
    }

    /// Sets the pole bone index.
    #[inline]
    pub fn pole_bone(mut self, bone: usize) -> Self {
        self.pole_bone = Some(bone);
        self
    }

    /// Sets whether or not to use the target bone's rotation.
    #[inline]
    pub fn use_target_rotation(mut self, value: bool) -> Self {
        self.use_target_rotation = value;
        self
    }
}
