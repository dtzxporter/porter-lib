use std::collections::BTreeMap;
use std::collections::HashMap;

use std::path::Path;

use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::model_file_type_cast;
use crate::model_file_type_fbx;
use crate::model_file_type_maya;
use crate::model_file_type_obj;
use crate::model_file_type_semodel;
use crate::model_file_type_smd;
use crate::model_file_type_xmodel_export;
use crate::model_file_type_xna_lara;
use crate::Aabb;
use crate::BlendShape;
use crate::Face;
use crate::FaceBuffer;
use crate::Material;
use crate::MaterialRemapFaces;
use crate::MaterialRemapVertices;
use crate::MaterialTextureRef;
use crate::Mesh;
use crate::ModelError;
use crate::ModelFileType;
use crate::Skeleton;
use crate::VertexBuffer;

/// A 3d model, with optional skeleton and materials.
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// The 3d skeleton for this model which can be empty.
    pub skeleton: Skeleton,
    /// The 3d meshes for this model which can be empty.
    pub meshes: Vec<Mesh>,
    /// A collection of blend shapes that go with this models meshes.
    pub blend_shapes: Vec<BlendShape>,
    /// A collection of materials for this model.
    pub materials: Vec<Material>,
}

impl Model {
    /// Constructs a new instance of model.
    pub fn new() -> Self {
        Self {
            skeleton: Skeleton::new(),
            meshes: Vec::new(),
            blend_shapes: Vec::new(),
            materials: Vec::new(),
        }
    }

    /// Constructs a new instance of model with the given capacity.
    pub fn with_capacity(bones: usize, meshes: usize) -> Self {
        Self {
            skeleton: Skeleton::with_capacity(bones),
            meshes: Vec::with_capacity(meshes),
            blend_shapes: Vec::new(),
            materials: Vec::new(),
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

        for blend_shape in &mut self.blend_shapes {
            blend_shape.scale(factor);
        }

        self.skeleton.scale(factor);
    }

    /// Transforms the model by the given matrix.
    pub fn transform(&mut self, matrix: &Matrix4x4) {
        for mesh in &mut self.meshes {
            mesh.transform(matrix);
        }

        self.skeleton.transform(matrix);
    }

    /// Remaps the model's meshes by their materials and faces.
    ///
    /// Note: This will reset the models blend shapes to ensure that they do not cause problems with the new meshes.
    pub fn remap_meshes_by_vertices<R: AsRef<[MaterialRemapVertices]>>(&mut self, remaps: R) {
        let remaps = remaps.as_ref();

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

        let mut new_meshes: HashMap<(String, bool, usize, usize), Mesh> =
            HashMap::with_capacity(remaps_per_material.len());

        for (material, opcodes) in remaps_per_material {
            let material_index = if let Some(index) = self
                .materials
                .iter()
                .position(|x| x.source_name == material)
            {
                index as isize
            } else {
                -1
            };

            for (mesh, verts) in opcodes {
                let old_mesh = &old_meshes[mesh];

                let new_mesh = new_meshes
                    .entry((
                        material.clone(),
                        old_mesh.vertices.colors(),
                        old_mesh.vertices.uv_layers(),
                        old_mesh.vertices.maximum_influence(),
                    ))
                    .or_insert(
                        Mesh::new(
                            FaceBuffer::new(),
                            VertexBuffer::builder()
                                .colors(old_mesh.vertices.colors())
                                .uv_layers(old_mesh.vertices.uv_layers())
                                .maximum_influence(old_mesh.vertices.maximum_influence())
                                .build(),
                        )
                        .name(old_mesh.name.clone()),
                    );

                if new_mesh.materials.is_empty() {
                    new_mesh.materials.push(material_index);
                }

                let mut vertex_remap: BTreeMap<u32, u32> = BTreeMap::new();
                let mut vertices: u32 = new_mesh.vertices.len() as u32;

                for (vertex_start, vertex_len) in verts {
                    for v in vertex_start..vertex_start + vertex_len {
                        let old_vertex = old_mesh.vertices.vertex(v);

                        new_mesh.vertices.create().copy_from(&old_vertex);

                        vertex_remap.insert(v as u32, vertices);

                        vertices += 1;
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
                        None => {
                            let old_vertex = old_mesh.vertices.vertex(face.i1 as usize);

                            new_mesh.vertices.create().copy_from(&old_vertex);

                            vertex_remap.insert(face.i1, vertices);

                            vertices += 1;

                            vertices - 1
                        }
                    };

                    let i2 = match i2 {
                        Some(i2) => i2,
                        None => {
                            let old_vertex = old_mesh.vertices.vertex(face.i2 as usize);

                            new_mesh.vertices.create().copy_from(&old_vertex);

                            vertex_remap.insert(face.i2, vertices);

                            vertices += 1;

                            vertices - 1
                        }
                    };

                    let i3 = match i3 {
                        Some(i3) => i3,
                        None => {
                            let old_vertex = old_mesh.vertices.vertex(face.i3 as usize);

                            new_mesh.vertices.create().copy_from(&old_vertex);

                            vertex_remap.insert(face.i3, vertices);

                            vertices += 1;

                            vertices - 1
                        }
                    };

                    new_mesh.faces.push(Face::new(i1, i2, i3));
                }
            }
        }

        for mesh in new_meshes.into_values() {
            self.meshes.push(mesh);
        }

        self.blend_shapes.clear();
    }

    /// Remaps the model's meshes by their materials and faces.
    ///
    /// Note: This will reset the models blend shapes to ensure that they do not cause problems with the new meshes.
    pub fn remap_meshes_by_faces<R: AsRef<[MaterialRemapFaces]>>(&mut self, remaps: R) {
        let remaps = remaps.as_ref();

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

        let mut new_meshes: HashMap<(String, bool, usize, usize), Mesh> =
            HashMap::with_capacity(remaps_per_material.len());

        for (material, opcodes) in remaps_per_material {
            let material_index = if let Some(index) = self
                .materials
                .iter()
                .position(|x| x.source_name == material)
            {
                index as isize
            } else {
                -1
            };

            for (mesh, faces) in opcodes {
                let old_mesh = &old_meshes[mesh];

                let new_mesh = new_meshes
                    .entry((
                        material.clone(),
                        old_mesh.vertices.colors(),
                        old_mesh.vertices.uv_layers(),
                        old_mesh.vertices.maximum_influence(),
                    ))
                    .or_insert(
                        Mesh::new(
                            FaceBuffer::new(),
                            VertexBuffer::builder()
                                .colors(old_mesh.vertices.colors())
                                .uv_layers(old_mesh.vertices.uv_layers())
                                .maximum_influence(old_mesh.vertices.maximum_influence())
                                .build(),
                        )
                        .name(old_mesh.name.clone()),
                    );

                if new_mesh.materials.is_empty() {
                    new_mesh.materials.push(material_index);
                }

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

                                let old_vertex = old_mesh.vertices.vertex(index as usize);

                                new_mesh.vertices.create().copy_from(&old_vertex);

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

        for mesh in new_meshes.into_values() {
            self.meshes.push(mesh);
        }

        self.blend_shapes.clear();
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
            ModelFileType::SEModel => model_file_type_semodel::to_semodel(path, self),
        }
    }

    /// Validates the model has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self) {
        self.skeleton.validate();

        for mesh in &self.meshes {
            mesh.validate(self.skeleton.bones.len());
        }
    }
}
