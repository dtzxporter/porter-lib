use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;

use porter_texture::Image;
use porter_texture::ImageFormat;

use crate::ViewportError;

/// A 3d mesh render material texture.
pub struct RenderMaterialTexture {
    bind_group: BindGroup,
}

/// Utility to allocate the fallback image for a material texture.
fn default_image() -> Result<Image, ViewportError> {
    let mut image =
        Image::new(4, 4, ImageFormat::R8G8B8A8Unorm).map_err(|_| ViewportError::InvalidAsset)?;

    image
        .create_frame()
        .map_err(|_| ViewportError::OutOfMemory)?
        .fill([161, 161, 161, 255]);

    Ok(image)
}

impl RenderMaterialTexture {
    /// Constructs a new render material texture from the given image, or defaults to a 4x4 gray square.
    pub fn from_image_default(
        instance: &GPUInstance,
        image: &Option<Image>,
        material_sampler: &Sampler,
        material_bind_group_layout: &BindGroupLayout,
    ) -> Result<Self, ViewportError> {
        let mut default: Option<Image> = None;

        if image.is_none() {
            default = Some(default_image()?);
        }

        let image = image
            .as_ref()
            .or(default.as_ref())
            // This ends up being a no-op because we've set default above.
            .unwrap();

        let format = image.format();

        if format.is_int() {
            return Err(ViewportError::Unsupported);
        }

        let Ok(format) = format.to_wgpu() else {
            return Err(ViewportError::Unsupported);
        };

        let texture_desc = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let Some(frame) = image.frames().first() else {
            return Err(ViewportError::InvalidAsset);
        };

        let texture = instance
            .device()
            .create_texture_with_data(
                instance.queue(),
                &texture_desc,
                TextureDataOrder::LayerMajor,
                frame.buffer(),
            );

        let texture_view = texture.create_view(&Default::default());

        let bind_group = instance
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: material_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(material_sampler),
                    },
                ],
            });

        Ok(Self { bind_group })
    }

    /// The bind group for this material texture.
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}
