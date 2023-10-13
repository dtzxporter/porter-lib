use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector3;

use porter_utils::SanitizeFilename;

/// Cleans a bone name.
fn sanitize_bone_name(name: String) -> String {
    let mut name = name.replace(' ', "_").sanitized();

    if name == "default" || name.is_empty() {
        name = String::from("_default");
    } else if name.as_bytes()[0].is_ascii_digit() {
        name = format!("_{}", name);
    }

    name
}

/// Represents a bone in a skeleton of a model.
#[derive(Debug, Clone)]
pub struct Bone {
    pub name: Option<String>,
    pub parent: i32,
    pub local_position: Option<Vector3>,
    pub local_rotation: Option<Quaternion>,
    pub local_scale: Option<Vector3>,
    pub world_position: Option<Vector3>,
    pub world_rotation: Option<Quaternion>,
    pub world_scale: Option<Vector3>,
}

impl Bone {
    /// Constructs a new instance of a bone.
    #[inline]
    pub fn new(name: Option<String>, parent: i32) -> Self {
        Self {
            name: name.map(sanitize_bone_name),
            parent,
            local_position: None,
            local_rotation: None,
            local_scale: None,
            world_position: None,
            world_rotation: None,
            world_scale: None,
        }
    }

    /// Sets the name.
    #[inline]
    pub fn name(mut self, name: Option<String>) -> Self {
        self.name = name.map(sanitize_bone_name);
        self
    }

    /// Sets the local position.
    #[inline]
    pub fn local_position(mut self, position: Vector3) -> Self {
        self.local_position = Some(position);
        self
    }

    /// Sets the local rotation.
    #[inline]
    pub fn local_rotation(mut self, rotation: Quaternion) -> Self {
        self.local_rotation = Some(rotation);
        self
    }

    /// Sets the scale.
    #[inline]
    pub fn local_scale(mut self, scale: Vector3) -> Self {
        self.local_scale = Some(scale);
        self
    }

    /// Sets the world position.
    #[inline]
    pub fn world_position(mut self, position: Vector3) -> Self {
        self.world_position = Some(position);
        self
    }

    /// Sets the world rotation.
    #[inline]
    pub fn world_rotation(mut self, rotation: Quaternion) -> Self {
        self.world_rotation = Some(rotation);
        self
    }

    /// Sets the world scale.
    #[inline]
    pub fn world_scale(mut self, scale: Vector3) -> Self {
        self.world_scale = Some(scale);
        self
    }

    /// Gets the local matrix (T * R * S).
    #[inline]
    pub fn local_matrix(&self) -> Matrix4x4 {
        Matrix4x4::create_position(self.local_position.unwrap_or_default())
            * self.local_rotation.unwrap_or_default().matrix4x4()
            * Matrix4x4::create_scale(self.local_scale.unwrap_or(Vector3::one()))
    }

    /// Gets the world matrix (T * R * S).
    pub fn world_matrix(&self) -> Matrix4x4 {
        Matrix4x4::create_position(self.world_position.unwrap_or_default())
            * self.world_rotation.unwrap_or_default().matrix4x4()
            * Matrix4x4::create_scale(self.world_scale.unwrap_or(Vector3::one()))
    }
}
