use crate::KeyframeValue;

/// A keyframe of a curve.
#[derive(Debug, Clone, Copy)]
pub struct Keyframe<V: KeyframeValue> {
    /// The value of the keyframe.
    pub value: V,
    /// The time at which this value takes place.
    pub time: u32,
}

impl<V> Keyframe<V>
where
    V: KeyframeValue,
{
    /// Linearly interpolates between two keyframes, at the given time.
    pub fn interpolate(&self, rhs: &Self, time: u32) -> Self {
        debug_assert!(self.time <= time && rhs.time >= time);

        let fraction = (time - self.time) as f32 / (rhs.time - self.time) as f32;

        Self {
            value: self
                .value
                .interpolate(&rhs.value, fraction),
            time,
        }
    }
}
