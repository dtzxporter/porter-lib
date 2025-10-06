use std::sync::Arc;

use wgpu::*;

use porter_gpu::GPUInstance;

use porter_model::Model;

use porter_texture::Image;

use crate::PreviewError;
use crate::RenderMaterialTexture;
use crate::RenderMesh;
use crate::RenderSkeleton;

/// A 3d render model.
pub struct RenderModel {
    meshes: Vec<RenderMesh>,
    skeleton: Option<RenderSkeleton>,
    srgb: bool,
}

impl RenderModel {
    /// Constructs a new render model from the given model.
    pub fn from_model(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        model: &Model,
        materials: &[Option<Image>],
        srgb: bool,
    ) -> Result<Self, PreviewError> {
        let materials: Vec<Arc<_>> = materials
            .iter()
            .map(|image| {
                RenderMaterialTexture::from_image_default(instance, image)
                    .or_else(|_| RenderMaterialTexture::from_image_default(instance, &None))
            })
            .chain([RenderMaterialTexture::from_image_default(instance, &None)])
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect();

        Ok(Self {
            meshes: model
                .meshes
                .iter()
                .map(|mesh| {
                    RenderMesh::from_mesh(instance, bind_group_layouts, mesh, &materials, true)
                })
                .chain(model.hairs.iter().map(|hair| {
                    RenderMesh::from_mesh(
                        instance,
                        bind_group_layouts,
                        &hair.to_mesh(),
                        &materials,
                        false,
                    )
                }))
                .collect::<Result<Vec<_>, _>>()?,
            skeleton: if model.skeleton.bones.is_empty() {
                None
            } else {
                Some(RenderSkeleton::from_skeleton(
                    instance,
                    bind_group_layouts,
                    &model.skeleton,
                ))
            },
            srgb,
        })
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

    /// Returns whether or not the models materials are in sRGB colorspace.
    pub fn srgb(&self) -> bool {
        self.srgb
    }

    /// Draws the model using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, show_bones: bool, wireframe: bool) {
        for mesh in &self.meshes {
            mesh.draw(render_pass, wireframe);
        }

        if show_bones && let Some(skeleton) = &self.skeleton {
            skeleton.draw(render_pass);
        }
    }
}
