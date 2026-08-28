use std::sync::Arc;

use wgpu::*;

use porter_gpu::GPUInstance;

use porter_model::Model;

use porter_texture::Image;

use crate::RenderMaterialTexture;
use crate::RenderMesh;
use crate::RenderSkeleton;
use crate::ViewportError;

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
        bind_group_layouts: &[Option<&BindGroupLayout>],
        model: &Model,
        materials: &[Option<Image>],
    ) -> Result<Self, ViewportError> {
        let srgb = materials
            .iter()
            .filter_map(|x| x.as_ref())
            .any(|x| x.format().is_srgb());

        let material_sampler = instance
            .device()
            .create_sampler(&SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..Default::default()
            });

        let material_bind_group_layout = instance
            .device()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let materials: Vec<Arc<_>> = materials
            .iter()
            .map(|image| {
                RenderMaterialTexture::from_image_default(
                    instance,
                    image,
                    &material_sampler,
                    &material_bind_group_layout,
                )
                .or_else(|_| {
                    RenderMaterialTexture::from_image_default(
                        instance,
                        &None,
                        &material_sampler,
                        &material_bind_group_layout,
                    )
                })
            })
            .chain([RenderMaterialTexture::from_image_default(
                instance,
                &None,
                &material_sampler,
                &material_bind_group_layout,
            )])
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(Arc::new)
            .collect();

        Ok(Self {
            meshes: model
                .meshes
                .iter()
                .map(|mesh| {
                    RenderMesh::from_mesh(
                        instance,
                        bind_group_layouts,
                        mesh,
                        &materials,
                        &material_bind_group_layout,
                        true,
                    )
                })
                .chain(model.hairs.iter().map(|hair| {
                    RenderMesh::from_mesh(
                        instance,
                        bind_group_layouts,
                        &hair.to_mesh(),
                        &materials,
                        &material_bind_group_layout,
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
        self.meshes
            .iter()
            .map(|mesh| mesh.vertex_count)
            .sum()
    }

    /// Returns the face count for this model.
    pub fn face_count(&self) -> usize {
        self.meshes
            .iter()
            .map(|mesh| mesh.face_count)
            .sum()
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
