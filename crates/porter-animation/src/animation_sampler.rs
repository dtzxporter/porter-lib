use std::collections::HashMap;

use porter_math::Angles;
use porter_math::Quaternion;
use porter_math::Vector3;

use crate::Animation;
use crate::AnimationError;
use crate::Constraint;
use crate::IkSolver;
use crate::Joint;
use crate::Keyframe;
use crate::KeyframeValue;
use crate::Keyframes;

/// Maximum number of joints the sampler supports.
const MAXIMUM_JOINTS: usize = 2048;

/// Bind position of a joint.
#[derive(Debug, Clone, Copy)]
struct BindPose {
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
}

/// A animation sampler that can evaluate joints and individual attributes.
#[derive(Debug)]
pub struct AnimationSampler {
    animation: Animation,
    frame_count: u32,
    frame_current: u32,
    constraints: Vec<Constraint>,
    joints: Vec<Joint>,
    joint_names: HashMap<String, usize>,
    joint_binds: Vec<BindPose>,
}

impl AnimationSampler {
    /// Constructs a new instance of animation sampler with the given animation.
    pub fn new(animation: Animation) -> Self {
        let frame_count = animation.frame_count();

        Self {
            animation,
            frame_count,
            frame_current: u32::MAX,
            constraints: Vec::new(),
            joints: Vec::new(),
            joint_names: HashMap::new(),
            joint_binds: Vec::new(),
        }
    }

    /// Gets the frame count.
    pub const fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Gets the current frame number we're on.
    pub const fn current_frame(&self) -> u32 {
        if self.frame_current == u32::MAX {
            0
        } else {
            self.frame_current
        }
    }

    /// Gets the frame rate the animation should play at.
    pub const fn frame_rate(&self) -> f32 {
        self.animation.frame_rate
    }

    /// Step to the next frame, wrapping back to 0 when at the end.
    pub fn step(&mut self) -> Result<(), AnimationError> {
        let mut reset_all_to_bind = false;

        if self.frame_current == u32::MAX || self.frame_current + 1 > self.frame_count {
            reset_all_to_bind = true;
            self.frame_current = 0;
        } else {
            self.frame_current += 1;
        }

        for (joint, bind_pose) in self
            .joints
            .iter_mut()
            .zip(self.joint_binds.iter())
        {
            if joint.reset_to_bind || reset_all_to_bind {
                joint.local_position = bind_pose.position;
                joint.local_rotation = bind_pose.rotation;
                joint.local_scale = bind_pose.scale;
            }
        }

        let current_frame = self.frame_current;

        for curve in &self.animation.curves {
            let Some(joint_index) = self.find_joint_index(curve.name()) else {
                continue;
            };

            let Some(joint) = self.joints.get_mut(joint_index) else {
                continue;
            };

            if let Keyframes::Translate(keyframes) = curve.keyframes()
                && let Some(keyframe) = interpolate(current_frame, keyframes)
            {
                joint.local_position = keyframe.value;
            } else if let Keyframes::Rotate(keyframes) = curve.keyframes()
                && let Some(keyframe) = interpolate(current_frame, keyframes)
            {
                joint.local_rotation = keyframe.value;
            } else if let Keyframes::Scale(keyframes) = curve.keyframes()
                && let Some(keyframe) = interpolate(current_frame, keyframes)
            {
                joint.local_scale = keyframe.value;
            }
        }

        self.calculate_transforms()?;
        self.calculate_constraints()?;

        Ok(())
    }

    /// Appends a joint to this sampler.
    pub fn push_joint(&mut self, name: Option<String>, joint: Joint) -> Result<(), AnimationError> {
        let index = self.joints.len();

        if index == MAXIMUM_JOINTS {
            return Err(AnimationError::JointsOverflow);
        }

        self.joint_binds.push(BindPose {
            position: joint.local_position,
            rotation: joint.local_rotation,
            scale: joint.local_scale,
        });

        if let Some(name) = name {
            self.joint_names.insert(name, index);
        }

        self.joints.push(joint);

        Ok(())
    }

    /// Finds the index of a join in this sampler by name.
    pub fn find_joint_index<N: AsRef<str>>(&self, name: N) -> Option<usize> {
        self.joint_names
            .get(name.as_ref())
            .copied()
    }

    /// Evaluates a given joint by it's name at the current time.
    pub fn evaulate_joint<N: AsRef<str>>(&self, name: N) -> Option<Joint> {
        self.joints
            .get(self.find_joint_index(name)?)
            .copied()
    }

    /// Evaluates a given joint by it's index at the current time.
    pub fn evaluate_joint_index(&self, index: usize) -> Option<Joint> {
        self.joints.get(index).copied()
    }

    /// Appends a constraint to this sampler.
    pub fn push_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Consumes the sampler, returning the inner animation.
    pub fn into_animation(self) -> Animation {
        self.animation
    }

    /// Calculates and applies constraints for each joint.
    fn calculate_constraints(&mut self) -> Result<(), AnimationError> {
        use Constraint::*;

        for i in 0..self.constraints.len() {
            match &self.constraints[i] {
                Length {
                    max,
                    source_joint,
                    target_joint,
                } => {
                    let source_joint = self
                        .evaulate_joint(source_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let target_index = self
                        .find_joint_index(target_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let mut target_joint = self
                        .evaluate_joint_index(target_index)
                        .ok_or(AnimationError::InvalidJointIndex)?;

                    let target = target_joint.world_position - source_joint.world_position;
                    let target_distance = target.length();

                    if target_distance > *max {
                        target_joint.world_position =
                            source_joint.world_position + target.normalized() * *max;
                    }

                    target_joint.generate_local_transforms(
                        &self
                            .evaluate_joint_index(target_joint.parent as usize)
                            .unwrap_or_default(),
                    );

                    self.joints[target_index] = target_joint;
                }
                Parent {
                    parent_joint,
                    target_joint,
                    position,
                    rotation,
                } => {
                    let parent_joint = self
                        .evaulate_joint(parent_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let target_index = self
                        .find_joint_index(target_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let mut target_joint = self
                        .evaluate_joint_index(target_index)
                        .ok_or(AnimationError::InvalidJointIndex)?;

                    let position = position.unwrap_or(target_joint.local_position);
                    let rotation = rotation.unwrap_or(target_joint.local_rotation);

                    target_joint.world_position = parent_joint.world_position
                        + position.transform(&parent_joint.world_rotation.to_4x4());
                    target_joint.world_rotation = parent_joint.world_rotation * rotation;

                    target_joint.generate_local_transforms(
                        &self
                            .evaluate_joint_index(target_joint.parent as usize)
                            .unwrap_or_default(),
                    );

                    self.joints[target_index] = target_joint;
                }
                Ik {
                    start_joint,
                    mid_joint,
                    end_joint,
                    handle,
                    pole_vector,
                    twist,
                    use_handle_rotation,
                } => {
                    let mut solver = IkSolver::new();

                    let start_joint_index = self
                        .find_joint_index(start_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let mut start_joint = self
                        .evaluate_joint_index(start_joint_index)
                        .ok_or(AnimationError::InvalidJointIndex)?;

                    let mid_joint_index = self
                        .find_joint_index(mid_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let mut mid_joint = self
                        .evaluate_joint_index(mid_joint_index)
                        .ok_or(AnimationError::InvalidJointIndex)?;

                    let end_joint_index = self
                        .find_joint_index(end_joint)
                        .ok_or(AnimationError::InvalidJointName)?;

                    let mut end_joint = self
                        .evaluate_joint_index(end_joint_index)
                        .ok_or(AnimationError::InvalidJointIndex)?;

                    solver.set_start_joint(start_joint.world_position);
                    solver.set_mid_joint(mid_joint.world_position);
                    solver.set_end_joint(end_joint.world_position);

                    let handle = self
                        .evaulate_joint(handle)
                        .ok_or(AnimationError::InvalidJointName)?;

                    solver.set_handle(handle.world_position);

                    if let Some(pole_vector) = pole_vector {
                        let pole_vector = self
                            .evaulate_joint(pole_vector)
                            .ok_or(AnimationError::InvalidJointName)?;

                        solver.set_pole_vector(pole_vector.world_position);
                    }

                    if let Some(twist) = twist {
                        let twist = self
                            .evaulate_joint(twist)
                            .ok_or(AnimationError::InvalidJointIndex)?;
                        let twist = twist
                            .world_rotation
                            .to_euler(Angles::Degrees);

                        solver.set_twist(twist.x, Angles::Degrees);
                    }

                    let (start_joint_solve, mid_joint_solve) = solver.solve();

                    mid_joint.world_rotation = mid_joint_solve * mid_joint.world_rotation;
                    mid_joint.generate_local_transforms(
                        &self
                            .evaluate_joint_index(mid_joint.parent as usize)
                            .unwrap_or_default(),
                    );

                    start_joint.world_rotation = start_joint_solve * start_joint.world_rotation;
                    start_joint.generate_local_transforms(
                        &self
                            .evaluate_joint_index(start_joint.parent as usize)
                            .unwrap_or_default(),
                    );

                    self.joints[mid_joint_index] = mid_joint;
                    self.joints[start_joint_index] = start_joint;

                    if *use_handle_rotation {
                        self.calculate_transforms()?;

                        let end_joint_parent = self
                            .evaluate_joint_index(end_joint.parent as usize)
                            .unwrap_or_default();

                        let local_rotation = end_joint_parent
                            .world_rotation
                            .inverse()
                            * handle.world_rotation;

                        end_joint.local_rotation = local_rotation;

                        self.joints[end_joint_index] = end_joint;
                    }
                }
            }

            self.calculate_transforms()?;
        }

        Ok(())
    }

    /// Calculates transforms for the joint cache.
    fn calculate_transforms(&mut self) -> Result<(), AnimationError> {
        let mut computed: [bool; MAXIMUM_JOINTS] = [false; MAXIMUM_JOINTS];

        if self.joints.len() > MAXIMUM_JOINTS {
            return Err(AnimationError::JointsOverflow);
        }

        for joint in 0..self.joints.len() {
            self.compute_world_transforms(joint, &mut computed);
        }

        Ok(())
    }

    /// Computes the world transforms for a joint in the cache.
    fn compute_world_transforms(&mut self, index: usize, computed: &mut [bool]) {
        let parent_index = self.joints[index].parent;

        if parent_index > -1 {
            let parent_index = parent_index as usize;

            if !computed[parent_index] {
                self.compute_world_transforms(parent_index, computed);
            }

            let (left, right) = self
                .joints
                .split_at_mut(index.max(parent_index));

            let (parent, joint) = if parent_index < index {
                let (_, after) = left.split_at_mut(parent_index);

                (&mut after[0], &mut right[0])
            } else {
                let (_, after) = left.split_at_mut(index);

                (&mut right[0], &mut after[0])
            };

            joint.generate_world_transforms(parent);
        } else {
            self.joints[index].generate_world_transforms(&Joint::new(-1));
        }

        computed[index] = true;
    }
}

/// Interpolates the keyframe for the given frame time.
#[inline]
fn interpolate<V: KeyframeValue>(frame: u32, keyframes: &[Keyframe<V>]) -> Option<Keyframe<V>> {
    let mut keyframe = keyframes.last().cloned()?;

    for window in keyframes.windows(2) {
        let keyframe0 = window[0];
        let keyframe1 = window[1];

        if !(keyframe0.time..keyframe1.time).contains(&frame) {
            continue;
        }

        keyframe = keyframe0.interpolate(&keyframe1, frame);
        break;
    }

    Some(keyframe)
}
