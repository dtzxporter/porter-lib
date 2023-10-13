use porter_model::MaterialTextureRefUsage;
use wgpu::*;

use porter_gpu::GPUInstance;
use porter_model::Model;
use porter_texture::Image;

use crate::RenderImage;
use crate::RenderMaterial;
use crate::RenderModel;

/// The type of 3d render asset to preview.
pub enum RenderType {
    Model(RenderModel),
    Image(RenderImage),
    Material(RenderMaterial),
}

pub trait ToRenderType {
    /// Converts an asset into a render type.
    fn to_render_type(
        &self,
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> RenderType;
}

impl ToRenderType for (Model, Vec<Option<Image>>) {
    fn to_render_type(
        &self,
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> RenderType {
        RenderType::Model(RenderModel::from_model(
            instance,
            bind_group_layouts,
            &self.0,
            &self.1,
        ))
    }
}

impl ToRenderType for Image {
    fn to_render_type(
        &self,
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> RenderType {
        RenderType::Image(RenderImage::from_image(instance, bind_group_layouts, self))
    }
}

impl ToRenderType for Vec<(MaterialTextureRefUsage, Image)> {
    fn to_render_type(
        &self,
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> RenderType {
        RenderType::Material(RenderMaterial::from_images(
            instance,
            bind_group_layouts,
            self,
        ))
    }
}
