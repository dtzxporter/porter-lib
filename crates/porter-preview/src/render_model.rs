use std::sync::Arc;

use wgpu::*;

use porter_gpu::GPUInstance;
use porter_model::Model;
use porter_texture::Image;

use crate::RenderMaterialTexture;
use crate::RenderMesh;
use crate::RenderSkeleton;

/// A 3d render model.
pub struct RenderModel {
    meshes: Vec<RenderMesh>,
    skeleton: Option<RenderSkeleton>,
}

impl RenderModel {
    /// Constructs a new render model from the given model.
    pub fn from_model(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        model: &Model,
        materials: &[Option<Image>],
    ) -> Self {
        let materials: Vec<Arc<_>> = materials
            .iter()
            .map(|image| RenderMaterialTexture::from_image_default(instance, image))
            .chain([RenderMaterialTexture::from_image_default(instance, &None)])
            .map(Arc::new)
            .collect();

        Self {
            meshes: model
                .meshes
                .iter()
                .map(|mesh| RenderMesh::from_mesh(instance, bind_group_layouts, mesh, &materials))
                .collect(),
            skeleton: if model.skeleton.is_empty() {
                None
            } else {
                Some(RenderSkeleton::from_skeleton(
                    instance,
                    bind_group_layouts,
                    &model.skeleton,
                ))
            },
        }
    }

    /// Returns the mesh count for this model.
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Returns the vertex count for this model.
    pub fn vertex_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.vertex_count).sum()
    }

    /// Returns the face count for this model.
    pub fn face_count(&self) -> usize {
        self.meshes.iter().map(|mesh| mesh.face_count).sum()
    }

    /// Returns the bone count for this model.
    pub fn bone_count(&self) -> usize {
        self.skeleton
            .as_ref()
            .map(|x| x.bone_count)
            .unwrap_or_default()
    }

    /// Draws the model using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, show_bones: bool, wireframe: bool) {
        for mesh in &self.meshes {
            mesh.draw(render_pass, wireframe);
        }

        if show_bones {
            if let Some(skeleton) = &self.skeleton {
                skeleton.draw(render_pass);
            }
        }
    }
}
