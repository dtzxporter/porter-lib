use std::collections::BTreeMap;
use std::collections::HashMap;

use std::path::Path;

use porter_math::Axis;
use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::Aabb;
use crate::BlendShape;
use crate::Face;
use crate::FaceBuffer;
use crate::Hair;
use crate::Material;
use crate::MaterialRemapFaces;
use crate::MaterialRemapVertices;
use crate::MaterialTextureRef;
use crate::Mesh;
use crate::ModelError;
use crate::ModelFileType;
use crate::Skeleton;
use crate::VertexBuffer;
use crate::WeightBoneId;
use crate::model_file_type_cast;
use crate::model_file_type_fbx;
use crate::model_file_type_maya;
use crate::model_file_type_obj;
use crate::model_file_type_smd;
use crate::model_file_type_xmodel_export;
use crate::model_file_type_xna_lara;

/// A 3d model, with optional skeleton and materials.
#[derive(Debug, Clone)]
pub struct Model {
    /// The 3d skeleton for this model which can be empty.
    pub skeleton: Skeleton,
    /// The 3d meshes for this model which can be empty.
    pub meshes: Vec<Mesh>,
    /// The 3d hairs for this model which can be empty.
    pub hairs: Vec<Hair>,
    /// A collection of materials for this model.
    pub materials: Vec<Material>,
    /// The up axis for this model.
    pub up_axis: Axis,
}

impl Model {
    /// Constructs a new instance of model.
    pub fn new() -> Self {
        Self {
            skeleton: Skeleton::new(),
            meshes: Vec::new(),
            hairs: Vec::new(),
            materials: Vec::new(),
            up_axis: Axis::Z,
        }
    }

    /// Constructs a new instance of model with the given capacity.
    pub fn with_capacity(bones: usize, meshes: usize) -> Self {
        Self {
            skeleton: Skeleton::with_capacity(bones),
            meshes: Vec::with_capacity(meshes),
            hairs: Vec::new(),
            materials: Vec::new(),
            up_axis: Axis::Z,
        }
    }

    /// Returns the total number of vertices in the model.
    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|x| x.vertices.len()).sum()
    }

    /// Returns the total number of faces in the model.
    pub fn face_count(&self) -> usize {
        self.meshes.iter().map(|x| x.faces.len()).sum()
    }

    /// Scales the model by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for mesh in &mut self.meshes {
            mesh.scale(factor);
        }

        for hair in &mut self.hairs {
            hair.scale(factor);
        }

        self.skeleton.scale(factor);
    }

    /// Transforms the model by the given matrix.
    pub fn transform(&mut self, matrix: &Matrix4x4) {
        for mesh in &mut self.meshes {
            mesh.transform(matrix);
        }

        for hair in &mut self.hairs {
            hair.transform(matrix);
        }

        self.skeleton.transform(matrix);
    }

    /// Applies a different bind pose to the model meshes.
    pub fn apply_bind_pose(&mut self, inv_bind_poses: &BTreeMap<WeightBoneId, Matrix4x4>) {
        for mesh in &mut self.meshes {
            mesh.apply_bind_pose(&self.skeleton, inv_bind_poses);
        }
    }

    /// Remaps the model's meshes by their materials and vertices.
    pub fn remap_meshes_by_vertices<R: AsRef<[MaterialRemapVertices]>>(&mut self, remaps: R) {
        let remaps = remaps.as_ref();

        if remaps.is_empty() {
            return;
        }

        #[allow(clippy::type_complexity)]
        let mut remaps_per_material: HashMap<String, BTreeMap<usize, Vec<(usize, usize)>>> =
            HashMap::new();

        for remap in remaps {
            remaps_per_material
                .entry(remap.material.clone())
                .or_default()
                .entry(remap.mesh)
                .or_default()
                .push((remap.vertex_start, remap.length));
        }

        let old_meshes = std::mem::take(&mut self.meshes);

        #[allow(clippy::type_complexity)]
        let mut new_meshes: HashMap<
            (String, usize, usize, usize),
            (Mesh, HashMap<String, BlendShape>),
        > = HashMap::with_capacity(remaps_per_material.len());

        for (material, opcodes) in remaps_per_material {
            let material_index = self
                .materials
                .iter()
                .position(|x| x.source_name == material);

            for (mesh, verts) in opcodes {
                let old_mesh = &old_meshes[mesh];

                let (new_mesh, new_shapes) = new_meshes
                    .entry((
                        material.clone(),
                        old_mesh.vertices.colors(),
                        old_mesh.vertices.uv_layers(),
                        old_mesh.vertices.maximum_influence(),
                    ))
                    .or_insert_with(|| {
                        let mesh = Mesh::with_skinning_method(
                            FaceBuffer::new(),
                            VertexBuffer::builder()
                                .colors(old_mesh.vertices.colors())
                                .uv_layers(old_mesh.vertices.uv_layers())
                                .maximum_influence(old_mesh.vertices.maximum_influence())
                                .build(),
                            old_mesh.skinning_method,
                        )
                        .name(old_mesh.name.clone());

                        (mesh, HashMap::with_capacity(old_mesh.blend_shapes.len()))
                    });

                new_mesh.material = material_index;

                let mut vertex_remap: BTreeMap<u32, u32> = BTreeMap::new();
                let mut vertices: u32 = new_mesh.vertices.len() as u32;

                let mut remap_index = |index: u32, vertex_remap: &mut BTreeMap<u32, u32>| {
                    vertex_remap.insert(index, vertices);

                    new_mesh
                        .vertices
                        .create()
                        .copy_from(&old_mesh.vertices.vertex(index as usize));

                    for blend_shape in &old_mesh.blend_shapes {
                        if let Some(delta) = blend_shape.vertex_deltas.get(&index) {
                            new_shapes
                                .entry(blend_shape.name.clone())
                                .or_insert_with(|| {
                                    BlendShape::new(blend_shape.name.clone())
                                        .target_scale(blend_shape.target_scale)
                                })
                                .vertex_deltas
                                .insert(vertices, *delta);
                        }
                    }

                    vertices += 1;
                    vertices - 1
                };

                for (vertex_start, vertex_len) in verts {
                    for v in vertex_start..vertex_start + vertex_len {
                        remap_index(v as u32, &mut vertex_remap);
                    }
                }

                for face in &old_mesh.faces {
                    let i1 = vertex_remap.get(&{ face.i1 }).copied();
                    let i2 = vertex_remap.get(&{ face.i2 }).copied();
                    let i3 = vertex_remap.get(&{ face.i3 }).copied();

                    if i1.is_none() && i2.is_none() && i3.is_none() {
                        continue;
                    }

                    let i1 = match i1 {
                        Some(i1) => i1,
                        None => remap_index(face.i1, &mut vertex_remap),
                    };

                    let i2 = match i2 {
                        Some(i2) => i2,
                        None => remap_index(face.i2, &mut vertex_remap),
                    };

                    let i3 = match i3 {
                        Some(i3) => i3,
                        None => remap_index(face.i3, &mut vertex_remap),
                    };

                    new_mesh.faces.push(Face::new(i1, i2, i3));
                }
            }
        }

        for (mut mesh, blend_shapes) in new_meshes.into_values() {
            for blend_shape in blend_shapes.into_values() {
                if !blend_shape.vertex_deltas.is_empty() {
                    mesh.blend_shapes.push(blend_shape);
                }
            }

            self.meshes.push(mesh);
        }
    }

    /// Remaps the model's meshes by their materials and faces.
    pub fn remap_meshes_by_faces<R: AsRef<[MaterialRemapFaces]>>(&mut self, remaps: R) {
        let remaps = remaps.as_ref();

        if remaps.is_empty() {
            return;
        }

        #[allow(clippy::type_complexity)]
        let mut remaps_per_material: HashMap<String, BTreeMap<usize, Vec<(usize, usize)>>> =
            HashMap::new();

        for remap in remaps {
            remaps_per_material
                .entry(remap.material.clone())
                .or_default()
                .entry(remap.mesh)
                .or_default()
                .push((remap.face_start, remap.length));
        }

        let old_meshes = std::mem::take(&mut self.meshes);

        #[allow(clippy::type_complexity)]
        let mut new_meshes: HashMap<
            (String, usize, usize, usize),
            (Mesh, HashMap<String, BlendShape>),
        > = HashMap::with_capacity(remaps_per_material.len());

        for (material, opcodes) in remaps_per_material {
            let material_index = self
                .materials
                .iter()
                .position(|x| x.source_name == material);

            for (mesh, faces) in opcodes {
                let old_mesh = &old_meshes[mesh];

                let (new_mesh, new_shapes) = new_meshes
                    .entry((
                        material.clone(),
                        old_mesh.vertices.colors(),
                        old_mesh.vertices.uv_layers(),
                        old_mesh.vertices.maximum_influence(),
                    ))
                    .or_insert_with(|| {
                        let mesh = Mesh::with_skinning_method(
                            FaceBuffer::new(),
                            VertexBuffer::builder()
                                .colors(old_mesh.vertices.colors())
                                .uv_layers(old_mesh.vertices.uv_layers())
                                .maximum_influence(old_mesh.vertices.maximum_influence())
                                .build(),
                            old_mesh.skinning_method,
                        )
                        .name(old_mesh.name.clone());

                        (mesh, HashMap::with_capacity(old_mesh.blend_shapes.len()))
                    });

                new_mesh.material = material_index;

                let mut vertex_remap: BTreeMap<u32, u32> = BTreeMap::new();
                let mut vertices: u32 = new_mesh.vertices.len() as u32;

                for (face_start, length) in faces {
                    for face in face_start..face_start + length {
                        let face = &old_mesh.faces[face];

                        let mut remap_index = |index: u32| {
                            if let Some(vertex) = vertex_remap.get(&index) {
                                *vertex
                            } else {
                                vertex_remap.insert(index, vertices);

                                new_mesh
                                    .vertices
                                    .create()
                                    .copy_from(&old_mesh.vertices.vertex(index as usize));

                                for blend_shape in &old_mesh.blend_shapes {
                                    if let Some(delta) = blend_shape.vertex_deltas.get(&index) {
                                        new_shapes
                                            .entry(blend_shape.name.clone())
                                            .or_insert_with(|| {
                                                BlendShape::new(blend_shape.name.clone())
                                                    .target_scale(blend_shape.target_scale)
                                            })
                                            .vertex_deltas
                                            .insert(vertices, *delta);
                                    }
                                }

                                vertices += 1;
                                vertices - 1
                            }
                        };

                        let i1 = remap_index(face.i1);
                        let i2 = remap_index(face.i2);
                        let i3 = remap_index(face.i3);

                        new_mesh.faces.push(Face::new(i1, i2, i3));
                    }
                }
            }
        }

        for (mut mesh, blend_shapes) in new_meshes.into_values() {
            for blend_shape in blend_shapes.into_values() {
                if !blend_shape.vertex_deltas.is_empty() {
                    mesh.blend_shapes.push(blend_shape);
                }
            }

            self.meshes.push(mesh);
        }
    }

    /// Gets the base texture for each material in this model.
    pub fn material_textures(&self) -> Vec<Option<MaterialTextureRef>> {
        let mut result = Vec::with_capacity(self.materials.len());

        for material in &self.materials {
            result.push(material.base_color_texture().cloned());
        }

        result
    }

    /// Calculates the bounding box for the given model.
    pub fn bounding_box(&self) -> Aabb {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;

        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for mesh in &self.meshes {
            for i in 0..mesh.vertices.len() {
                let position = mesh.vertices.vertex(i).position();

                min_x = min_x.min(position.x);
                min_y = min_y.min(position.y);
                min_z = min_z.min(position.z);
                max_x = max_x.max(position.x);
                max_y = max_y.max(position.y);
                max_z = max_z.max(position.z);
            }
        }

        Aabb::new(
            Vector3::new(min_x, min_y, min_z),
            Vector3::new(max_x, max_y, max_z),
        )
    }

    /// Saves the model to the given file path in the given model format.
    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
        file_type: ModelFileType,
    ) -> Result<(), ModelError> {
        match file_type {
            ModelFileType::Obj => model_file_type_obj::to_obj(path, self),
            ModelFileType::Smd => model_file_type_smd::to_smd(path, self),
            ModelFileType::XnaLara => model_file_type_xna_lara::to_xna_lara(path, self),
            ModelFileType::XModelExport => {
                model_file_type_xmodel_export::to_xmodel_export(path, self)
            }
            ModelFileType::Cast => model_file_type_cast::to_cast(path, self),
            ModelFileType::Fbx => model_file_type_fbx::to_fbx(path, self),
            ModelFileType::Maya => model_file_type_maya::to_maya(path, self),
        }
    }

    /// Validates the model has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self) {
        self.skeleton.validate();

        for mesh in &self.meshes {
            mesh.validate(self.skeleton.bones.len());
        }

        for hair in &self.hairs {
            hair.validate();
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
