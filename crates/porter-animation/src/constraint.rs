use porter_math::Quaternion;
use porter_math::Vector3;

/// Represents the type of constraint used for an animation.
#[derive(Debug)]
pub enum Constraint {
    #[doc(hidden)]
    Length {
        max: f32,
        source_joint: String,
        target_joint: String,
    },
    #[doc(hidden)]
    Parent {
        parent_joint: String,
        target_joint: String,
        position: Option<Vector3>,
        rotation: Option<Quaternion>,
    },
    #[doc(hidden)]
    Ik {
        start_joint: String,
        mid_joint: String,
        end_joint: String,
        handle: String,
        pole_vector: Option<String>,
        twist: Option<String>,
        use_handle_rotation: bool,
    },
}

impl Constraint {
    /// Constructs a new length constraint.
    pub fn length(max: f32, source_joint: &str, target_joint: &str) -> Self {
        Self::Length {
            max,
            source_joint: source_joint.to_owned(),
            target_joint: target_joint.to_owned(),
        }
    }

    /// Constructs a new parent constraint with the given offsets.
    pub fn parent(
        parent_joint: &str,
        target_joint: &str,
        position: Vector3,
        rotation: Quaternion,
    ) -> Self {
        Self::Parent {
            parent_joint: parent_joint.to_owned(),
            target_joint: target_joint.to_owned(),
            position: Some(position),
            rotation: Some(rotation),
        }
    }

    /// Constructs a new parent constraint that maintains the target offsets.
    pub fn parent_maintain_offset(parent_joint: &str, target_joint: &str) -> Self {
        Self::Parent {
            parent_joint: parent_joint.to_owned(),
            target_joint: target_joint.to_owned(),
            position: None,
            rotation: None,
        }
    }

    /// Constructs a new inverse kinematics constraint.
    pub fn ik(
        start_joint: &str,
        mid_joint: &str,
        end_joint: &str,
        handle: &str,
        pole_vector: Option<&str>,
        twist: Option<&str>,
        use_handle_rotation: bool,
    ) -> Self {
        Self::Ik {
            start_joint: start_joint.to_owned(),
            mid_joint: mid_joint.to_owned(),
            end_joint: end_joint.to_owned(),
            handle: handle.to_owned(),
            pole_vector: pole_vector.map(ToOwned::to_owned),
            twist: twist.map(ToOwned::to_owned),
            use_handle_rotation,
        }
    }
}
