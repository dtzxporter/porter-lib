use std::fmt::Debug;

use porter_math::Quaternion;
use porter_math::Vector3;

use crate::Keyframe;
use crate::Keyframes;

/// The value of a keyframe.
pub trait KeyframeValue: Debug + Clone + Copy {
    /// Inserts the value into a keyframe collection at the given time.
    fn insert(self, time: u32, keyframes: &mut Keyframes);
    /// Linearly interpolates between two keyframes, at the given time.
    fn interpolate(&self, rhs: &Self, time: f32) -> Self;
}

impl KeyframeValue for Quaternion {
    fn insert(self, time: u32, keyframes: &mut Keyframes) {
        if let Keyframes::Rotate(keyframes) = keyframes {
            keyframes.push(Keyframe { value: self, time });
        }
    }

    fn interpolate(&self, rhs: &Self, time: f32) -> Self {
        self.slerp(*rhs, time)
    }
}

impl KeyframeValue for Vector3 {
    fn insert(self, time: u32, keyframes: &mut Keyframes) {
        if let Keyframes::Translate(keyframes) | Keyframes::Scale(keyframes) = keyframes {
            keyframes.push(Keyframe { value: self, time });
        }
    }

    fn interpolate(&self, rhs: &Self, time: f32) -> Self {
        self.lerp(*rhs, time)
    }
}

impl KeyframeValue for f32 {
    fn insert(self, time: u32, keyframes: &mut Keyframes) {
        if let Keyframes::BlendShape(keyframes) = keyframes {
            keyframes.push(Keyframe { value: self, time });
        }
    }

    fn interpolate(&self, rhs: &Self, time: f32) -> Self {
        self + (rhs - self) * time
    }
}

impl KeyframeValue for bool {
    fn insert(self, time: u32, keyframes: &mut Keyframes) {
        if let Keyframes::Visibility(keyframes) = keyframes {
            keyframes.push(Keyframe { value: self, time });
        }
    }

    fn interpolate(&self, rhs: &Self, time: f32) -> Self {
        if time >= 1.0 { *rhs } else { *self }
    }
}

impl KeyframeValue for () {
    fn insert(self, time: u32, keyframes: &mut Keyframes) {
        if let Keyframes::Notetrack(keyframes) = keyframes {
            keyframes.push(Keyframe { value: (), time });
        }
    }

    fn interpolate(&self, _rhs: &Self, _time: f32) -> Self {}
}
