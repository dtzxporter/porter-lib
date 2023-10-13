use std::path::Path;

use crate::model_file_type_cast;
use crate::model_file_type_fbx;
use crate::model_file_type_maya;
use crate::model_file_type_obj;
use crate::model_file_type_semodel;
use crate::model_file_type_smd;
use crate::model_file_type_xmodel_export;
use crate::model_file_type_xna_lara;
use crate::BlendShape;
use crate::Material;
use crate::MaterialTextureRef;
use crate::Mesh;
use crate::ModelError;
use crate::ModelFileType;
use crate::Skeleton;

/// A 3d model, with optional skeleton and materials.
#[derive(Debug, Clone, Default)]
pub struct Model {
    pub skeleton: Skeleton,
    pub meshes: Vec<Mesh>,
    pub blend_shapes: Vec<BlendShape>,
    pub materials: Vec<Material>,
}

impl Model {
    /// Constructs a new instance of model.
    #[inline]
    pub fn new() -> Self {
        Self {
            skeleton: Skeleton::new(),
            meshes: Vec::new(),
            blend_shapes: Vec::new(),
            materials: Vec::new(),
        }
    }

    /// Constructs a new instance of model with the given capacity.
    #[inline]
    pub fn with_capacity(bones: usize, meshes: usize) -> Self {
        Self {
            skeleton: Skeleton::with_capacity(bones),
            meshes: Vec::with_capacity(meshes),
            blend_shapes: Vec::new(),
            materials: Vec::new(),
        }
    }

    /// Returns the total number of vertices in the model.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|x| x.vertices.len()).sum()
    }

    /// Returns the total number of faces in the model.
    #[inline]
    pub fn face_count(&self) -> usize {
        self.meshes.iter().map(|x| x.faces.len()).sum()
    }

    /// Scales the model by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for mesh in &mut self.meshes {
            mesh.scale(factor);
        }

        self.skeleton.scale(factor);
    }

    /// Gets the base texture for each material in this model.
    pub fn material_textures(&self) -> Vec<Option<MaterialTextureRef>> {
        let mut result = Vec::with_capacity(self.materials.len());

        for material in &self.materials {
            result.push(material.base_texture().cloned());
        }

        result
    }

    /// Calculates the bounding box for the given model, [mins; maxs;].
    pub fn bounding_box(&self) -> [f32; 6] {
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

        [min_x, min_y, min_z, max_x, max_y, max_z]
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
            ModelFileType::SEModel => model_file_type_semodel::to_semodel(path, self),
            ModelFileType::Cast => model_file_type_cast::to_cast(path, self),
            ModelFileType::Fbx => model_file_type_fbx::to_fbx(path, self),
            ModelFileType::Maya => model_file_type_maya::to_maya(path, self),
        }
    }
}
