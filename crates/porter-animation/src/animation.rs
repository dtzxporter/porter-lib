use std::path::Path;

use porter_math::Axis;

use crate::AnimationError;
use crate::AnimationFileType;
use crate::Curve;
use crate::CurveAttribute;
use crate::CurveDataType;
use crate::CurveModeOverride;
use crate::KeyframeValue;
use crate::animation_file_type_cast;
use crate::animation_file_type_seanim;

// A 3d animation.
#[derive(Debug, Clone)]
pub struct Animation {
    /// The framerate this animation should play at.
    pub framerate: f32,
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
    /// Constructs a new animation with the given framerate.
    pub fn new(framerate: f32, looping: bool) -> Self {
        Self {
            framerate,
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
            AnimationFileType::SEAnim => animation_file_type_seanim::to_seanim(path, self),
            AnimationFileType::Cast => animation_file_type_cast::to_cast(path, self),
        }
    }

    /// Returns the most common curve data type.
    pub fn average_data_type(&self) -> CurveDataType {
        let mut data_types: [usize; 3] = [0, 0, 0];

        for curve in &self.curves {
            match curve.data_type() {
                CurveDataType::Absolute => data_types[0] += 1,
                CurveDataType::Additive => data_types[1] += 1,
                CurveDataType::Relative => data_types[2] += 1,
            }
        }

        if data_types[1] > data_types[0] && data_types[1] > data_types[2] {
            CurveDataType::Additive
        } else if data_types[2] > data_types[0] && data_types[2] > data_types[1] {
            CurveDataType::Relative
        } else {
            CurveDataType::Absolute
        }
    }

    /// Returns the length of the animation in frames.
    pub fn frame_count(&self) -> u32 {
        let mut result = 0;

        for curve in &self.curves {
            for keyframe in curve.keyframes() {
                result = result.max(keyframe.time);
            }
        }

        // Frame count is the length of the animation in frames
        // Frames start at index 0, so we add one to get the count
        result + 1
    }

    /// Returns the total count of notifications in this animation.
    pub fn notification_count(&self) -> usize {
        self.curves
            .iter()
            .filter(|x| matches!(x.attribute(), CurveAttribute::Notetrack))
            .map(|x| x.len())
            .sum()
    }

    /// Scales this animation by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for curve in &mut self.curves {
            if matches!(curve.attribute(), CurveAttribute::Translate) {
                for keyframe in curve.keyframes_mut() {
                    if let KeyframeValue::Vector3(vector) = &mut keyframe.value {
                        *vector *= factor;
                    }
                }
            }
        }
    }
}
