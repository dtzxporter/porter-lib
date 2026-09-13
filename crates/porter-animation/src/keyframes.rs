use porter_math::Quaternion;
use porter_math::Vector3;

use crate::AnimationError;
use crate::CurveAttribute;
use crate::Keyframe;
use crate::KeyframeValue;

/// A collection of keyframes for curve attributes.
#[derive(Debug, Clone)]
pub enum Keyframes {
    /// Translation keyframes.
    Translate(Vec<Keyframe<Vector3>>),
    /// Rotation keyframes.
    Rotate(Vec<Keyframe<Quaternion>>),
    /// Scale keyframes.
    Scale(Vec<Keyframe<Vector3>>),
    /// Visibility keyframes.
    Visibility(Vec<Keyframe<bool>>),
    /// Notetrack keyframes.
    Notetrack(Vec<Keyframe<()>>),
    /// Blend shape keyframes.
    BlendShape(Vec<Keyframe<f32>>),
}

impl Keyframes {
    /// Internal method, see [`Curve::new`](crate::Curve::new).
    pub(crate) const fn new(attribute: CurveAttribute) -> Self {
        use CurveAttribute::*;

        match attribute {
            Translate => Keyframes::Translate(Vec::new()),
            Rotate => Keyframes::Rotate(Vec::new()),
            Scale => Keyframes::Scale(Vec::new()),
            Visibility => Keyframes::Visibility(Vec::new()),
            Notetrack => Keyframes::Notetrack(Vec::new()),
            BlendShape => Keyframes::BlendShape(Vec::new()),
        }
    }

    /// Internal method, see [`Curve::attribute`](crate::Curve::attribute).
    pub(crate) const fn attribute(&self) -> CurveAttribute {
        use CurveAttribute::*;

        match &self {
            Keyframes::Translate(_) => Translate,
            Keyframes::Rotate(_) => Rotate,
            Keyframes::Scale(_) => Scale,
            Keyframes::Visibility(_) => Visibility,
            Keyframes::Notetrack(_) => Notetrack,
            Keyframes::BlendShape(_) => BlendShape,
        }
    }

    /// Internal method, see [`Curve::largest_frame_time`](crate::Curve::largest_frame_time).
    pub(crate) fn largest_frame_time(&self) -> u32 {
        use Keyframes::*;

        match self {
            Translate(keyframes) => max_time(keyframes),
            Rotate(keyframes) => max_time(keyframes),
            Scale(keyframes) => max_time(keyframes),
            Visibility(keyframes) => max_time(keyframes),
            Notetrack(keyframes) => max_time(keyframes),
            BlendShape(keyframes) => max_time(keyframes),
        }
    }

    /// Internal method, see [`Curve::sort`](crate::Curve::sort).
    pub(crate) fn sort(&mut self) {
        use Keyframes::*;

        match self {
            Translate(keyframes) => sort_time(keyframes),
            Rotate(keyframes) => sort_time(keyframes),
            Scale(keyframes) => sort_time(keyframes),
            Visibility(keyframes) => sort_time(keyframes),
            Notetrack(keyframes) => sort_time(keyframes),
            BlendShape(keyframes) => sort_time(keyframes),
        }
    }

    /// Internal method, see [`Curve::len`](crate::Curve::len).
    pub(crate) const fn len(&self) -> usize {
        use Keyframes::*;

        match self {
            Translate(keyframes) => keyframes.len(),
            Rotate(keyframes) => keyframes.len(),
            Scale(keyframes) => keyframes.len(),
            Visibility(keyframes) => keyframes.len(),
            Notetrack(keyframes) => keyframes.len(),
            BlendShape(keyframes) => keyframes.len(),
        }
    }

    /// Internal method, see [`Curve::is_empty`](crate::Curve::is_empty).
    pub(crate) const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Internal method, see [`Curve::try_reserve`](crate::Curve::try_reserve).
    pub(crate) fn try_reserve(&mut self, additional: usize) -> Result<(), AnimationError> {
        use Keyframes::*;

        match self {
            Translate(keyframes) => try_reserve(keyframes, additional),
            Rotate(keyframes) => try_reserve(keyframes, additional),
            Scale(keyframes) => try_reserve(keyframes, additional),
            Visibility(keyframes) => try_reserve(keyframes, additional),
            Notetrack(keyframes) => try_reserve(keyframes, additional),
            BlendShape(keyframes) => try_reserve(keyframes, additional),
        }
    }

    /// Internal method, see [`Curve::try_reserve_exact`](crate::Curve::try_reserve_exact).
    pub(crate) fn try_reserve_exact(&mut self, additional: usize) -> Result<(), AnimationError> {
        use Keyframes::*;

        match self {
            Translate(keyframes) => try_reserve_exact(keyframes, additional),
            Rotate(keyframes) => try_reserve_exact(keyframes, additional),
            Scale(keyframes) => try_reserve_exact(keyframes, additional),
            Visibility(keyframes) => try_reserve_exact(keyframes, additional),
            Notetrack(keyframes) => try_reserve_exact(keyframes, additional),
            BlendShape(keyframes) => try_reserve_exact(keyframes, additional),
        }
    }
}

/// Utility method to get the maximum time from a set of keyframes.
#[inline]
fn max_time<V: KeyframeValue>(keyframes: &[Keyframe<V>]) -> u32 {
    keyframes
        .iter()
        .map(|keyframe| keyframe.time)
        .max()
        .unwrap_or(0)
}

/// Utility method to sort a set of keyframes by time.
#[inline]
fn sort_time<V: KeyframeValue>(keyframes: &mut [Keyframe<V>]) {
    keyframes.sort_by_key(|keyframe| keyframe.time);
}

/// Utility method to try and reserve `additional` keyframes.
#[inline]
fn try_reserve<V: KeyframeValue>(
    keyframes: &mut Vec<Keyframe<V>>,
    additional: usize,
) -> Result<(), AnimationError> {
    Ok(keyframes.try_reserve(additional)?)
}

/// Utility method to try and reserve exactly `additional` keyframes.
#[inline]
fn try_reserve_exact<V: KeyframeValue>(
    keyframes: &mut Vec<Keyframe<V>>,
    additional: usize,
) -> Result<(), AnimationError> {
    Ok(keyframes.try_reserve_exact(additional)?)
}
