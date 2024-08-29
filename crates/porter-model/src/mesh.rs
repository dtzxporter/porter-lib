use std::collections::BTreeMap;

use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::BlendShape;
use crate::FaceBuffer;
use crate::Skeleton;
use crate::SkinningMethod;
use crate::VertexBuffer;
use crate::WeightBoneId;

/// A polygon mesh for a model.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name of this mesh, used mostly for blend shape targets.
    pub name: Option<String>,
    /// The face buffer for this mesh.
    pub faces: FaceBuffer,
    /// The vertex buffer for this mesh.
    pub vertices: VertexBuffer,
    /// The material index for this mesh.
    pub material: Option<usize>,
    /// A collection of blend shapes that go with this mesh.
    pub blend_shapes: Vec<BlendShape>,
    /// The method used to skin this mesh.
    pub skinning_method: SkinningMethod,
}

impl Mesh {
    /// Constructs a new mesh instance.
    pub fn new(faces: FaceBuffer, vertices: VertexBuffer) -> Self {
        Self {
            name: None,
            material: None,
            faces,
            vertices,
            blend_shapes: Vec::new(),
            skinning_method: SkinningMethod::Linear,
        }
    }

    /// Constructs a new mesh instance with the given skinning method.
    pub fn with_skinning_method(
        faces: FaceBuffer,
        vertices: VertexBuffer,
        skinning_method: SkinningMethod,
    ) -> Self {
        Self {
            name: None,
            material: None,
            faces,
            vertices,
            blend_shapes: Vec::new(),
            skinning_method,
        }
    }

    /// Sets an optional name for this mesh.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.name = name.map(|x| x.into());
        self
    }

    /// Scales the mesh by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for i in 0..self.vertices.len() {
            let mut vertex = self.vertices.vertex_mut(i);

            vertex.set_position(vertex.position() * factor);
        }

        for blend_shape in &mut self.blend_shapes {
            blend_shape.scale(factor);
        }
    }

    /// Transforms the mesh by the given matrix.
    pub fn transform(&mut self, matrix: &Matrix4x4) {
        let normal = matrix.to_3x3().to_4x4().inverse().transpose();

        for i in 0..self.vertices.len() {
            let mut vertex = self.vertices.vertex_mut(i);

            vertex.set_position(vertex.position().transform(matrix));
            vertex.set_normal(vertex.normal().transform(&normal).normalized());
        }

        for blend_shape in &mut self.blend_shapes {
            blend_shape.transform(matrix)
        }

        if matrix.determinant() < 0.0 {
            for face in &mut self.faces {
                face.swap_order();
            }
        }
    }

    /// Applies a different bind pose to the mesh.
    pub fn apply_bind_pose(
        &mut self,
        skeleton: &Skeleton,
        inv_bind_poses: &BTreeMap<WeightBoneId, Matrix4x4>,
    ) {
        let maximum_influence = self.vertices.maximum_influence();

        if maximum_influence == 0 {
            return;
        }

        for v in 0..self.vertices.len() {
            let mut vertex = self.vertices.vertex_mut(v);

            let mut position = Vector3::zero();
            let mut normal = Vector3::zero();

            for w in 0..maximum_influence {
                let weight = vertex.weight(w);

                let inv_bind_pose = inv_bind_poses
                    .get(&{ weight.bone })
                    .copied()
                    .unwrap_or_default();

                let transform = skeleton.bones[weight.bone as usize].world_matrix() * inv_bind_pose;
                let transform_normal = transform.to_3x3().to_4x4();

                position += vertex.position().transform(&transform) * weight.value;
                normal += vertex.normal().transform(&transform_normal) * weight.value;
            }

            vertex.set_position(position);
            vertex.set_normal(normal.normalized());
        }
    }

    /// Validates the mesh has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self, bone_count: usize) {
        for v in 0..self.vertices.len() {
            let vertex = self.vertices.vertex(v);

            let normal = vertex.normal();
            let length_squared = normal.length_squared();

            if !(0.98..=1.025).contains(&length_squared) {
                println!(
                    "Validation Error: Found normal with non-1.0 square sum: {} - {}, {}, {} [{}]",
                    length_squared, normal.x, normal.y, normal.z, v
                );
            }

            let mut total = 0.0;

            for w in 0..self.vertices.maximum_influence() {
                let weight = vertex.weight(w);

                if weight.bone >= bone_count as WeightBoneId {
                    println!(
                        "Validation Error: Found weight bone outside of skeleton: {} [{}:{}]",
                        { weight.bone },
                        v,
                        w
                    );
                }

                total += weight.value;
            }

            if self.vertices.maximum_influence() > 0 && !(0.9825..=1.125).contains(&total) {
                println!(
                    "Validation Error: Found weight doesn't add up to 1.0: {} [{}]",
                    total, v
                );
            }
        }

        let vertex_count = self.vertices.len() as u32;

        for (index, face) in self.faces.iter().enumerate() {
            if face.i1 < vertex_count && face.i2 < vertex_count && face.i3 < vertex_count {
                continue;
            }

            println!(
                "Validate Error: Found face with invalid indexes: {}, {}, {} [{}]",
                { face.i1 },
                { face.i2 },
                { face.i3 },
                index
            );
        }

        for (index, face) in self.faces.iter().enumerate() {
            let degenerate = face.i1 == face.i2 || face.i1 == face.i3 || face.i2 == face.i3;

            if !degenerate {
                continue;
            }

            println!(
                "Validate Error: Found face with degenerate tris: {}, {}, {} [{}]",
                { face.i1 },
                { face.i2 },
                { face.i3 },
                index
            );
        }

        for blend_shape in self.blend_shapes.iter() {
            if blend_shape.vertex_deltas.len() > self.vertices.len() {
                println!(
                    "Validate Error: Found invalid blend shape too much data: {:?}",
                    blend_shape.name
                );
                continue;
            }

            for (index, vertex_index) in blend_shape.vertex_deltas.keys().enumerate() {
                if *vertex_index as usize >= self.vertices.len() {
                    println!(
                        "Validate Error: Found invalid blend shape index: {:?} [{}:{}]",
                        vertex_index, blend_shape.name, index
                    );
                }
            }
        }
    }
}
