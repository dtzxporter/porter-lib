use std::path::Path;

use porter_math::Axis;

use crate::AnimationError;
use crate::AnimationFileType;
use crate::Curve;
use crate::CurveAttribute;
use crate::CurveModeOverride;
use crate::Keyframes;
use crate::animation_file_type_cast;

/// A 3d animation.
#[derive(Debug, Clone)]
pub struct Animation {
    /// The frame rate this animation should play at.
    pub frame_rate: f32,
    /// Whether or not this animation should loop.
    pub looping: bool,
    /// A collection of curves for this animation.
    pub curves: Vec<Curve>,
    /// A collection of curve datatype overrides for this animation.
    pub curve_mode_overrides: Vec<CurveModeOverride>,
    /// The up axis for this animation.
    pub up_axis: Axis,
}

impl Animation {
    /// Constructs a new animation with the given frame rate.
    pub const fn new(frame_rate: f32, looping: bool) -> Self {
        Self {
            frame_rate,
            looping,
            curves: Vec::new(),
            curve_mode_overrides: Vec::new(),
            up_axis: Axis::Z,
        }
    }

    /// Saves the animation to the given file path in the given animation format.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: AnimationFileType,
    ) -> Result<(), AnimationError> {
        match file_type {
            AnimationFileType::Cast => animation_file_type_cast::to_cast(path, self),
        }
    }

    /// Attempts to find a curve with the given name and attribute.
    pub fn find<N: AsRef<str>>(&self, name: N, attribute: CurveAttribute) -> Option<&Curve> {
        self.curves
            .get(self.index(name, attribute)?)
    }

    /// Attempts to find a mutable curve with the given name and attribute.
    pub fn find_mut<N: AsRef<str>>(
        &mut self,
        name: N,
        attribute: CurveAttribute,
    ) -> Option<&mut Curve> {
        let index = self.index(name, attribute)?;

        self.curves.get_mut(index)
    }

    /// Attempts to find the index of the curve with the given name and attribute.
    pub fn index<N: AsRef<str>>(&self, name: N, attribute: CurveAttribute) -> Option<usize> {
        self.curves
            .iter()
            .position(|curve| curve.name() == name.as_ref() && curve.attribute() == attribute)
    }

    /// Returns the length of the animation in frames.
    pub fn frame_count(&self) -> u32 {
        let mut result = 0;

        for curve in &self.curves {
            result = result.max(curve.largest_frame_time());
        }

        // Frame count is the length of the animation in frames
        // Frames start at index 0, so we add one to get the count
        result + 1
    }

    /// Scales this animation by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for curve in &mut self.curves {
            if let Keyframes::Translate(keyframes) = curve.keyframes_mut() {
                for keyframe in keyframes {
                    keyframe.value *= factor;
                }
            }
        }
    }
}
