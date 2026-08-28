use crate::Animation;
use crate::AnimationError;
use crate::AnimationSampler;
use crate::Curve;
use crate::CurveAttribute;
use crate::CurveDataType;

/// Support for baking new animation curves.
pub struct AnimationCompiler {
    sampler: AnimationSampler,
    curves: Vec<Curve>,
}

impl AnimationCompiler {
    /// Constructs a new animation compiler with the given sampler.
    pub const fn new(sampler: AnimationSampler) -> Self {
        Self {
            sampler,
            curves: Vec::new(),
        }
    }

    /// Appends a new curve to be compiled.
    pub fn push<N: Into<String>>(mut self, name: N, attribute: CurveAttribute) -> Self {
        self.curves
            .push(Curve::new(name, attribute, CurveDataType::Absolute));
        self
    }

    /// Compiles the new curves and produces an animation.
    pub fn compile(mut self) -> Result<Animation, AnimationError> {
        let frame_count = self.sampler.frame_count();

        for curve in &mut self.curves {
            curve.try_reserve_exact(frame_count as _)?;
        }

        for frame in 0..frame_count {
            self.sampler.step()?;

            for curve in &mut self.curves {
                let joint = self
                    .sampler
                    .evaulate_joint(curve.name())
                    .ok_or(AnimationError::InvalidJointName)?;

                match curve.attribute() {
                    CurveAttribute::Translate => curve.insert(frame, joint.local_position),
                    CurveAttribute::Rotation => curve.insert(frame, joint.local_rotation),
                    CurveAttribute::Scale => curve.insert(frame, joint.local_scale),
                    _ => {
                        // Not supported yet.
                    }
                }
            }
        }

        let mut animation = self.sampler.into_animation();

        animation
            .curves
            // Add the compiled curves to the animation.
            .extend(self.curves);

        Ok(animation)
    }
}
