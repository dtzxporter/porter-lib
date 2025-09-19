use porter_cast::CastId;
use porter_cast::CastNode;
use porter_cast::CastPropertyId;
use porter_cast::CastPropertyValue;

use porter_math::Quaternion;
use porter_math::Vector3;

/// An instance of a scene in a 3d world.
#[derive(Debug, Clone)]
pub struct Instance {
    pub name: Option<String>,
    pub reference: String,
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl Instance {
    /// Constructs a new instance.
    pub fn new(name: Option<String>, reference: String) -> Self {
        Self {
            name,
            reference,
            position: Vector3::zero(),
            rotation: Quaternion::identity(),
            scale: Vector3::one(),
        }
    }

    /// Scales the instance by the given factor.
    pub fn scale(&mut self, factor: f32) {
        self.position *= factor;
    }

    /// Sets the position of this instance.
    pub fn set_position(mut self, position: Vector3) -> Self {
        self.position = position;
        self
    }

    /// Sets the rotation of this instance.
    pub fn set_rotation(mut self, rotation: Quaternion) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the scale of this instance.
    pub fn set_scale(mut self, scale: Vector3) -> Self {
        self.scale = scale;
        self
    }

    /// Saves the instance to the cast node.
    pub(crate) fn save(&self, node: &mut CastNode) {
        let instance_node = node.create(CastId::Instance);

        if let Some(name) = &self.name {
            instance_node
                .create_property(CastPropertyId::String, "n")
                .push(name.as_str());
        }

        instance_node
            .create_property(CastPropertyId::Vector3, "p")
            .push(self.position);

        instance_node
            .create_property(CastPropertyId::Vector4, "r")
            .push(self.rotation);

        instance_node
            .create_property(CastPropertyId::Vector3, "s")
            .push(self.scale);

        let file_node = instance_node.create(CastId::File);

        file_node
            .create_property(CastPropertyId::String, "p")
            .push(self.reference.as_str());

        let file_hash = CastPropertyValue::from(file_node);

        instance_node
            .create_property(CastPropertyId::Integer64, "rf")
            .push(file_hash);
    }
}
