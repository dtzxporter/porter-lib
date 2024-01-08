use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::Bone;
use crate::Constraint;
use crate::ConstraintType;
use crate::IKHandle;

/// Represents a skeleton, or collection of bones for a model.
#[derive(Debug, Clone, Default)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub ik_handles: Vec<IKHandle>,
    pub constraints: Vec<Constraint>,
}

impl Skeleton {
    /// Constructs a new skeleton.
    #[inline]
    pub fn new() -> Self {
        Self {
            bones: Vec::new(),
            ik_handles: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Constructs a new skeleton with the given capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bones: Vec::with_capacity(capacity),
            ik_handles: Vec::new(),
            constraints: Vec::new(),
        }
    }

    /// Generates local transforms based on available global transforms.
    pub fn generate_local_transforms(&mut self) {
        for i in 0..self.bones.len() {
            if self.bones[i].parent > -1 {
                let parent_matrix = (!self.bones[self.bones[i].parent as usize]
                    .world_rotation
                    .unwrap_or_default())
                .matrix4x4();

                self.bones[i].local_position = Some(
                    (self.bones[i].world_position.unwrap_or_default()
                        - self.bones[self.bones[i].parent as usize]
                            .world_position
                            .unwrap_or_default())
                    .transform(&parent_matrix),
                );

                self.bones[i].local_rotation = Some(
                    self.bones[!self.bones[i].parent as usize]
                        .world_rotation
                        .unwrap_or_default()
                        * self.bones[i].world_rotation.unwrap_or_default(),
                );

                self.bones[i].local_scale = Some(
                    self.bones[!self.bones[i].parent as usize]
                        .world_scale
                        .unwrap_or(Vector3::one())
                        * self.bones[i].local_scale.unwrap_or(Vector3::one()),
                );
            } else {
                self.bones[i].local_position = self.bones[i].world_position;
                self.bones[i].local_rotation = self.bones[i].world_rotation;
                self.bones[i].local_scale = self.bones[i].world_scale;
            }
        }
    }

    /// Generates world transforms based on local transforms.
    pub fn generate_world_transforms(&mut self) {
        for i in 0..self.bones.len() {
            if self.bones[i].parent > -1 {
                let parent_index = self.bones[i].parent as usize;
                let parent_world = self.bones[parent_index].world_matrix();
                let parent_position = self.bones[parent_index].world_position.unwrap_or_default();
                let local_position =
                    Matrix4x4::create_position(self.bones[i].local_position.unwrap_or_default());

                let result = ((parent_world * local_position) * parent_world.inverse()).position();

                self.bones[i].world_position = Some(Vector3::new(
                    parent_position.x + result.x,
                    parent_position.y + result.y,
                    parent_position.z + result.z,
                ));

                self.bones[i].world_rotation = Some(
                    self.bones[self.bones[i].parent as usize]
                        .world_rotation
                        .unwrap_or_default()
                        * self.bones[i].local_rotation.unwrap_or_default(),
                );

                self.bones[i].world_scale = Some(
                    self.bones[self.bones[i].parent as usize]
                        .world_scale
                        .unwrap_or(Vector3::one())
                        * self.bones[i].local_scale.unwrap_or(Vector3::one()),
                );
            } else {
                self.bones[i].world_position = self.bones[i].local_position;
                self.bones[i].world_rotation = self.bones[i].local_rotation;
                self.bones[i].world_scale = self.bones[i].local_scale;
            }
        }
    }

    /// Scales the skeleton by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for bone in &mut self.bones {
            if let Some(position) = &mut bone.local_position {
                *position *= factor;
            }
            if let Some(position) = &mut bone.world_position {
                *position *= factor;
            }
        }
    }

    /// Creates an ik handle if the all of given bones are found in the skeleton.
    #[allow(clippy::too_many_arguments)]
    pub fn create_ik_handle<
        S: AsRef<str>,
        E: AsRef<str>,
        T: AsRef<str>,
        P: AsRef<str>,
        V: AsRef<str>,
    >(
        &mut self,
        name: Option<String>,
        start_bone: S,
        end_bone: E,
        target_bone: T,
        pole_bone: Option<P>,
        pole_vector_bone: Option<V>,
        use_target_rotation: bool,
    ) {
        let start_bone = self.index(start_bone);
        let end_bone = self.index(end_bone);
        let target_bone = self.index(target_bone);

        if let (Some(start_bone), Some(end_bone), Some(target_bone)) =
            (start_bone, end_bone, target_bone)
        {
            let mut handle = IKHandle::new(name, start_bone, end_bone)
                .target_bone(target_bone)
                .use_target_rotation(use_target_rotation);

            let pole_bone = if let Some(pole_bone) = pole_bone {
                self.index(pole_bone)
            } else {
                None
            };

            let pole_vector_bone = if let Some(pole_vector_bone) = pole_vector_bone {
                self.index(pole_vector_bone)
            } else {
                None
            };

            if let Some(pole_bone) = pole_bone {
                handle = handle.pole_bone(pole_bone);
            }

            if let Some(pole_vector_bone) = pole_vector_bone {
                handle = handle.pole_vector_bone(pole_vector_bone);
            }

            self.ik_handles.push(handle);
        }
    }

    /// Creates a constraint if all of the given bones are found in the skeleton.
    pub fn create_constraint<C: AsRef<str>, T: AsRef<str>>(
        &mut self,
        name: Option<String>,
        constraint_type: ConstraintType,
        constraint_bone: C,
        target_bone: T,
        maintain_offset: bool,
        axis_to_skip: &'static str,
    ) {
        let constraint_bone = self.index(constraint_bone);
        let target_bone = self.index(target_bone);

        if let (Some(constraint_bone), Some(target_bone)) = (constraint_bone, target_bone) {
            let mut constraint = Constraint::new(
                name,
                constraint_type,
                constraint_bone,
                target_bone,
                maintain_offset,
            );

            if axis_to_skip.contains('x') {
                constraint = constraint.skip_x(true);
            }

            if axis_to_skip.contains('y') {
                constraint = constraint.skip_y(true);
            }

            if axis_to_skip.contains('z') {
                constraint = constraint.skip_z(true);
            }

            self.constraints.push(constraint);
        }
    }

    /// Attempts to find the index of the bone with the given name.
    pub fn index<N: AsRef<str>>(&self, bone: N) -> Option<usize> {
        self.bones.iter().position(|x| {
            if let Some(name) = &x.name {
                name == bone.as_ref()
            } else {
                false
            }
        })
    }

    /// Validates the skeleton has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self) {
        for (index, bone) in self.bones.iter().enumerate() {
            if bone.parent == -1 || (bone.parent > -1 && bone.parent < self.bones.len() as i32) {
                continue;
            }

            println!(
                "Validate Error: Found bone with invalid parent: {} [{}] {:?}",
                bone.parent, index, bone.name
            );
        }
    }
}
