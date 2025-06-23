use porter_math::Quaternion;
use porter_math::Vector3;

/// A keyframe value of a curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeValue {
    Vector3(Vector3),
    Quaternion(Quaternion),
    Bool(bool),
    Float(f32),
    None,
}

/// A keyframe of a curve.
#[derive(Debug, Clone, Copy)]
pub struct Keyframe {
    /// The value of the keyframe.
    pub value: KeyframeValue,
    /// The time at which this value takes place.
    pub time: u32,
}

impl From<Vector3> for KeyframeValue {
    fn from(value: Vector3) -> Self {
        Self::Vector3(value)
    }
}

impl From<Quaternion> for KeyframeValue {
    fn from(value: Quaternion) -> Self {
        Self::Quaternion(value)
    }
}

impl From<bool> for KeyframeValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f32> for KeyframeValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<()> for KeyframeValue {
    fn from(_: ()) -> Self {
        Self::None
    }
}
