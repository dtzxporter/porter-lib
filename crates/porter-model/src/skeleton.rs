use std::fmt;
use std::ops;

use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::Bone;

/// Represents a skeleton, or collection of bones for a model.
#[derive(Clone, Default)]
pub struct Skeleton {
    inner: Vec<Bone>,
}

impl Skeleton {
    /// Constructs a new skeleton.
    #[inline]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Constructs a new skeleton with the given capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Generates local transforms based on available global transforms.
    pub fn generate_local_transforms(&mut self) {
        for i in 0..self.inner.len() {
            if self.inner[i].parent > -1 {
                let parent_matrix = (!self.inner[self.inner[i].parent as usize]
                    .world_rotation
                    .unwrap_or_default())
                .matrix4x4();

                self.inner[i].local_position = Some(
                    (self.inner[i].world_position.unwrap_or_default()
                        - self.inner[self.inner[i].parent as usize]
                            .world_position
                            .unwrap_or_default())
                    .transform(&parent_matrix),
                );

                self.inner[i].local_rotation = Some(
                    self.inner[!self.inner[i].parent as usize]
                        .world_rotation
                        .unwrap_or_default()
                        * self.inner[i].world_rotation.unwrap_or_default(),
                );

                self.inner[i].local_scale = Some(
                    self.inner[!self.inner[i].parent as usize]
                        .world_scale
                        .unwrap_or(Vector3::one())
                        * self.inner[i].local_scale.unwrap_or(Vector3::one()),
                );
            } else {
                self.inner[i].local_position = self.inner[i].world_position;
                self.inner[i].local_rotation = self.inner[i].world_rotation;
                self.inner[i].local_scale = self.inner[i].world_scale;
            }
        }
    }

    /// Generates world transforms based on local transforms.
    pub fn generate_world_transforms(&mut self) {
        for i in 0..self.inner.len() {
            if self.inner[i].parent > -1 {
                let parent_index = self.inner[i].parent as usize;
                let parent_world = self.inner[parent_index].world_matrix();
                let parent_position = self.inner[parent_index].world_position.unwrap_or_default();
                let local_position =
                    Matrix4x4::create_position(self.inner[i].local_position.unwrap_or_default());

                let result = ((parent_world * local_position) * parent_world.inverse()).position();

                self.inner[i].world_position = Some(Vector3::new(
                    parent_position.x + result.x,
                    parent_position.y + result.y,
                    parent_position.z + result.z,
                ));

                self.inner[i].world_rotation = Some(
                    self.inner[self.inner[i].parent as usize]
                        .world_rotation
                        .unwrap_or_default()
                        * self.inner[i].local_rotation.unwrap_or_default(),
                );

                self.inner[i].world_scale = Some(
                    self.inner[self.inner[i].parent as usize]
                        .world_scale
                        .unwrap_or(Vector3::one())
                        * self.inner[i].local_scale.unwrap_or(Vector3::one()),
                );
            } else {
                self.inner[i].world_position = self.inner[i].local_position;
                self.inner[i].world_rotation = self.inner[i].local_rotation;
                self.inner[i].world_scale = self.inner[i].local_scale;
            }
        }
    }

    /// Scales the skeleton by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for bone in &mut self.inner {
            if let Some(position) = &mut bone.local_position {
                *position *= factor;
            }
            if let Some(position) = &mut bone.world_position {
                *position *= factor;
            }
        }
    }
}

impl ops::Deref for Skeleton {
    type Target = Vec<Bone>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for Skeleton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ops::Index<usize> for Skeleton {
    type Output = Bone;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl ops::IndexMut<usize> for Skeleton {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl fmt::Debug for Skeleton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.inner.iter()).finish()
    }
}
