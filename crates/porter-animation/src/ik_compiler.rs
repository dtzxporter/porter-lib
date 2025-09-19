use porter_math::Angles;

use crate::Animation;
use crate::AnimationError;
use crate::AnimationSampler;
use crate::Curve;
use crate::CurveAttribute;
use crate::CurveDataType;
use crate::IkSolver;

/// Support for baking a 3d animation with inverse kinematics.
#[derive(Debug)]
pub struct IkCompiler {
    sampler: AnimationSampler,
    start_joint: String,
    mid_joint: String,
    end_joint: String,
    handle: String,
    pole_vector: Option<String>,
    twist: Option<String>,
    use_handle_rotation: bool,
}

impl IkCompiler {
    /// Constructs a new ikcompiler from the given animation sampler.
    pub fn new(sampler: AnimationSampler) -> Self {
        Self {
            sampler,
            start_joint: String::new(),
            mid_joint: String::new(),
            end_joint: String::new(),
            handle: String::new(),
            pole_vector: None,
            twist: None,
            use_handle_rotation: false,
        }
    }

    /// Sets the first joint in the ik chain.
    pub fn start_joint<N: Into<String>>(mut self, joint: N) -> Self {
        self.start_joint = joint.into();
        self
    }

    /// Sets the middle joint in the ik chain.
    pub fn mid_joint<N: Into<String>>(mut self, joint: N) -> Self {
        self.mid_joint = joint.into();
        self
    }

    /// Sets the end joint in the ik chain.
    pub fn end_joint<N: Into<String>>(mut self, joint: N) -> Self {
        self.end_joint = joint.into();
        self
    }

    /// Sets the end joint to inherit the rotation of the handle in the ik chain.
    pub fn use_handle_rotation(mut self) -> Self {
        self.use_handle_rotation = true;
        self
    }

    /// Sets the handle (target) for the ik chain.
    pub fn handle<N: Into<String>>(mut self, joint: N) -> Self {
        self.handle = joint.into();
        self
    }

    /// Sets the pole vector for the ik chain.
    pub fn pole_vector<N: Into<String>>(mut self, joint: N) -> Self {
        self.pole_vector = Some(joint.into());
        self
    }

    /// Sets the twist for the ik chain.
    pub fn twist<N: Into<String>>(mut self, joint: N) -> Self {
        self.twist = Some(joint.into());
        self
    }

    /// Compile keyframes for this inverse kinematics animation to a regular animation.
    pub fn compile(mut self) -> Result<Animation, AnimationError> {
        let mut solver = IkSolver::new();

        let mut start_joint_curve = Curve::new(
            &self.start_joint,
            CurveAttribute::Rotation,
            CurveDataType::Absolute,
        );

        let mut mid_joint_curve = Curve::new(
            &self.mid_joint,
            CurveAttribute::Rotation,
            CurveDataType::Absolute,
        );

        let mut end_joint_curve = Curve::new(
            &self.end_joint,
            CurveAttribute::Rotation,
            CurveDataType::Absolute,
        );

        let frame_count = self.sampler.frame_count();

        for frame in 0..frame_count {
            self.sampler.step()?;

            let start_joint = self
                .sampler
                .evaulate_joint(&self.start_joint)
                .ok_or(AnimationError::InvalidJointName)?;

            let mid_joint = self
                .sampler
                .evaulate_joint(&self.mid_joint)
                .ok_or(AnimationError::InvalidJointName)?;

            let end_joint = self
                .sampler
                .evaulate_joint(&self.end_joint)
                .ok_or(AnimationError::InvalidJointName)?;

            solver.set_start_joint(start_joint.world_position);
            solver.set_mid_joint(mid_joint.world_position);
            solver.set_end_joint(end_joint.world_position);

            let handle = self
                .sampler
                .evaulate_joint(&self.handle)
                .ok_or(AnimationError::InvalidJointName)?;

            solver.set_handle(handle.world_position);

            if let Some(pole_vector) = &self.pole_vector {
                let pole_vector = self
                    .sampler
                    .evaulate_joint(pole_vector)
                    .ok_or(AnimationError::InvalidJointName)?;

                solver.set_pole_vector(pole_vector.world_position);
            }

            // TODO: We should support more ways to input twist, optionally a float parameter too.

            if let Some(twist) = &self.twist {
                let twist = self
                    .sampler
                    .evaulate_joint(twist)
                    .ok_or(AnimationError::InvalidJointName)?;
                let twist = twist.world_rotation.to_euler(Angles::Degrees);

                solver.set_twist(twist.x, Angles::Degrees);
            }

            let (start_joint_solution, mid_joint_solution) = solver.solve();

            let mut mid_joint_solve = mid_joint;

            mid_joint_solve.world_rotation = mid_joint_solution * mid_joint_solve.world_rotation;
            mid_joint_solve.generate_local_transforms(
                &self
                    .sampler
                    .evaluate_joint_index(mid_joint_solve.parent as usize)
                    .unwrap_or_default(),
            );

            let mut start_joint_solve = start_joint;

            start_joint_solve.world_rotation =
                start_joint_solution * start_joint_solve.world_rotation;
            start_joint_solve.generate_local_transforms(
                &self
                    .sampler
                    .evaluate_joint_index(start_joint_solve.parent as usize)
                    .unwrap_or_default(),
            );

            let mut end_joint_solve = end_joint;

            if self.use_handle_rotation {
                self.sampler.replace_joint(&self.mid_joint, mid_joint_solve);
                self.sampler
                    .replace_joint(&self.start_joint, start_joint_solve);

                self.sampler.refresh();

                let end_joint_parent = self
                    .sampler
                    .evaluate_joint_index(end_joint_solve.parent as usize)
                    .unwrap_or_default();

                let local_rotation =
                    end_joint_parent.world_rotation.inverse() * handle.world_rotation;

                end_joint_solve.local_rotation = local_rotation;

                self.sampler.replace_joint(&self.start_joint, start_joint);
                self.sampler.replace_joint(&self.mid_joint, mid_joint);
            }

            start_joint_curve.insert(frame, start_joint_solve.local_rotation);
            mid_joint_curve.insert(frame, mid_joint_solve.local_rotation);
            end_joint_curve.insert(frame, end_joint_solve.local_rotation);
        }

        let mut animation = self.sampler.into_animation();

        animation
            .curves
            .extend([start_joint_curve, mid_joint_curve, end_joint_curve]);

        Ok(animation)
    }
}
