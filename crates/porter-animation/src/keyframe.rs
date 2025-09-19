use porter_math::Quaternion;
use porter_math::Vector3;

use crate::AnimationError;

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

impl Keyframe {
    /// Linearly interpolates between two keyframes, at the given time.
    pub fn lerp(&self, rhs: &Self, time: u32) -> Self {
        debug_assert!(self.time <= time && rhs.time >= time);

        let fraction = (time - self.time) as f32 / (rhs.time - self.time) as f32;

        Self {
            value: self.value.lerp(&rhs.value, fraction),
            time,
        }
    }
}

impl KeyframeValue {
    /// Linearly interpolates between two keyframes, at the given time.
    pub fn lerp(&self, rhs: &Self, time: f32) -> Self {
        debug_assert!(matches!(
            (self, rhs),
            (Self::Vector3(_), Self::Vector3(_))
                | (Self::Quaternion(_), Self::Quaternion(_))
                | (Self::Bool(_), Self::Bool(_))
                | (Self::Float(_), Self::Float(_))
        ));

        match (self, rhs) {
            (Self::Vector3(lhs), Self::Vector3(rhs)) => Self::Vector3(lhs.lerp(*rhs, time)),
            (Self::Quaternion(lhs), Self::Quaternion(rhs)) => {
                Self::Quaternion(lhs.slerp(*rhs, time))
            }
            (Self::Bool(lhs), Self::Bool(rhs)) => {
                if time >= 1.0 {
                    Self::Bool(*rhs)
                } else {
                    Self::Bool(*lhs)
                }
            }
            (Self::Float(lhs), Self::Float(rhs)) => Self::Float(lhs + (rhs - lhs) * time),
            _ => Self::None,
        }
    }
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

impl TryFrom<KeyframeValue> for f32 {
    type Error = AnimationError;

    #[inline]
    fn try_from(value: KeyframeValue) -> Result<Self, Self::Error> {
        if let KeyframeValue::Float(value) = value {
            Ok(value)
        } else {
            Err(AnimationError::InvalidKeyframeValue)
        }
    }
}

impl TryFrom<KeyframeValue> for bool {
    type Error = AnimationError;

    #[inline]
    fn try_from(value: KeyframeValue) -> Result<Self, Self::Error> {
        if let KeyframeValue::Bool(value) = value {
            Ok(value)
        } else {
            Err(AnimationError::InvalidKeyframeValue)
        }
    }
}

impl TryFrom<KeyframeValue> for Vector3 {
    type Error = AnimationError;

    #[inline]
    fn try_from(value: KeyframeValue) -> Result<Self, Self::Error> {
        if let KeyframeValue::Vector3(value) = value {
            Ok(value)
        } else {
            Err(AnimationError::InvalidKeyframeValue)
        }
    }
}

impl TryFrom<KeyframeValue> for Quaternion {
    type Error = AnimationError;

    #[inline]
    fn try_from(value: KeyframeValue) -> Result<Self, Self::Error> {
        if let KeyframeValue::Quaternion(value) = value {
            Ok(value)
        } else {
            Err(AnimationError::InvalidKeyframeValue)
        }
    }
}
