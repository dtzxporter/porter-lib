use std::collections::HashMap;
use std::collections::HashSet;

use porter_utils::HashExt;

use crate::Animation;
use crate::AnimationError;
use crate::CurveAttribute;
use crate::Joint;
use crate::Keyframe;
use crate::KeyframeValue;

/// Key used for the sampler frame cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct SampleKey(u64);

impl SampleKey {
    /// Constructs a new instance of sample key with the given name and attribute.
    pub fn new<N: AsRef<str>>(name: N, attribute: CurveAttribute) -> Self {
        let mut hash: [u8; 16] = [0; 16];

        hash[0..8].copy_from_slice(&name.as_ref().hash_murmura64().to_le_bytes());
        hash[8..16].copy_from_slice(&(attribute as u64).to_le_bytes());

        Self((&hash[0..]).hash_murmura64())
    }
}

/// A animation sampler that can evaluate joints and individual attributes.
#[derive(Debug)]
pub struct AnimationSampler {
    animation: Animation,
    frame_count: u32,
    frame_current: u32,
    frame_cache: HashMap<SampleKey, KeyframeValue>,
    joint_cache: Vec<Joint>,
    joint_names: Vec<Option<String>>,
}

impl AnimationSampler {
    /// Constructs a new instance of animation sampler with the given animation.
    pub fn new(animation: Animation) -> Self {
        let frame_count = animation.frame_count();

        Self {
            animation,
            frame_count,
            frame_current: u32::MAX,
            frame_cache: HashMap::new(),
            joint_cache: Vec::new(),
            joint_names: Vec::new(),
        }
    }

    /// Gets the frame count.
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Gets the current frame number we're on.
    pub fn current_frame(&self) -> u32 {
        if self.frame_current == u32::MAX {
            0
        } else {
            self.frame_current
        }
    }

    /// Step to the next frame, wrapping back to 0 when at the end.
    pub fn step(&mut self) -> Result<(), AnimationError> {
        if self.frame_current == u32::MAX || self.frame_current + 1 > self.frame_count {
            self.frame_current = 0;
        } else {
            self.frame_current += 1;
        }

        for curve in &self.animation.curves {
            debug_assert!(curve.keyframes().is_sorted_by_key(|curve| curve.time));

            if curve.is_empty() {
                continue;
            }

            let mut keyframe: Keyframe = curve
                .keyframes() // We must have at least one keyframe because we made sure it's not empty.
                .last()
                .cloned()
                .unwrap();

            for window in curve.keyframes().windows(2) {
                let keyframe0 = window[0];
                let keyframe1 = window[1];

                if !(keyframe0.time..keyframe1.time).contains(&self.frame_current) {
                    continue;
                }

                keyframe = keyframe0.lerp(&keyframe1, self.frame_current);
                break;
            }

            self.frame_cache.insert(
                SampleKey::new(curve.name(), curve.attribute()),
                keyframe.value,
            );
        }

        for (joint, joint_name) in self.joint_cache.iter_mut().zip(self.joint_names.iter()) {
            if let Some(name) = joint_name {
                if let Some(translation) = self
                    .frame_cache
                    .get(&SampleKey::new(name, CurveAttribute::Translate))
                {
                    joint.local_position = translation.to_owned().try_into()?;
                }

                if let Some(rotation) = self
                    .frame_cache
                    .get(&SampleKey::new(name, CurveAttribute::Rotation))
                {
                    joint.local_rotation = rotation.to_owned().try_into()?;
                }
            }
        }

        self.refresh();

        Ok(())
    }

    /// Appends a joint to this sampler.
    pub fn push_joint(&mut self, name: Option<String>, joint: Joint) {
        self.joint_names.push(name);
        self.joint_cache.push(joint);
    }

    /// Replaces an existing join in this sampler.
    pub fn replace_joint<N: AsRef<str>>(&mut self, name: N, joint: Joint) {
        let Some(index) = self.joint_names.iter().position(|joint_name| {
            if let Some(joint_name) = joint_name {
                joint_name == name.as_ref()
            } else {
                false
            }
        }) else {
            return;
        };

        if let Some(old_joint) = self.joint_cache.get_mut(index) {
            *old_joint = joint;
        }
    }

    /// Evaluates a given joint by it's name at the current time.
    pub fn evaulate_joint<N: AsRef<str>>(&self, name: N) -> Option<Joint> {
        let index = self.joint_names.iter().position(|joint_name| {
            if let Some(joint_name) = joint_name {
                joint_name == name.as_ref()
            } else {
                false
            }
        })?;

        self.joint_cache.get(index).copied()
    }

    /// Evaluates a given joint by it's index at the current time.
    pub fn evaluate_joint_index(&self, index: usize) -> Option<Joint> {
        self.joint_cache.get(index).copied()
    }

    /// Evaulates a named attribute at the current time.
    pub fn evaulate<N: AsRef<str>>(
        &self,
        name: N,
        attribute: CurveAttribute,
    ) -> Option<KeyframeValue> {
        self.frame_cache
            .get(&SampleKey::new(name, attribute))
            .copied()
    }

    /// Refreshes the animations joint transform cache.
    pub fn refresh(&mut self) {
        let mut computed: HashSet<usize> = HashSet::new();

        for joint in 0..self.joint_cache.len() {
            self.compute_world_transforms(joint, &mut computed);
        }
    }

    /// Consumes the sampler, returning the inner animation.
    pub fn into_animation(self) -> Animation {
        self.animation
    }

    /// Computes world transforms for the joint cache.
    fn compute_world_transforms(&mut self, index: usize, computed: &mut HashSet<usize>) {
        if computed.contains(&index) {
            return;
        }

        let parent_index = self.joint_cache[index].parent;

        if parent_index > -1 {
            let parent_index = parent_index as usize;

            if !computed.contains(&parent_index) {
                self.compute_world_transforms(parent_index, computed);
            }

            let (left, right) = self.joint_cache.split_at_mut(index.max(parent_index));

            let (parent, joint) = if parent_index < index {
                let (_, after) = left.split_at_mut(parent_index);

                (&mut after[0], &mut right[0])
            } else {
                let (_, after) = left.split_at_mut(index);

                (&mut right[0], &mut after[0])
            };

            joint.generate_world_transforms(parent);
        } else {
            self.joint_cache[index].generate_world_transforms(&Joint::new(-1));
        }

        computed.insert(index);
    }
}
