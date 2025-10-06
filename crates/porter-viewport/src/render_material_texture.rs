use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;

use porter_texture::Image;
use porter_texture::ImageFormat;

use porter_utils::AsThisSlice;

use crate::PreviewError;

/// A 3d mesh render material texture.
pub struct RenderMaterialTexture {
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

/// Utility to allocate the fallback image for a material texture.
fn default_image() -> Result<Image, PreviewError> {
    let mut image =
        Image::new(4, 4, ImageFormat::R8G8B8A8Unorm).map_err(|_| PreviewError::InvalidAsset)?;

    image
        .create_frame()
        .map_err(|_| PreviewError::OutOfMemory)?
        .buffer_mut()
        .copy_from_slice([0xFFA1A1A1u32; 4 * 4].as_slice().as_this_slice());

    Ok(image)
}

impl RenderMaterialTexture {
    /// Constructs a new render material texture from the given image, or defaults to a 4x4 gray square.
    pub fn from_image_default(
        instance: &GPUInstance,
        image: &Option<Image>,
    ) -> Result<Self, PreviewError> {
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
            return Err(PreviewError::Unsupported);
        }

        let Ok(format) = format.to_wgpu() else {
            return Err(PreviewError::Unsupported);
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
            return Err(PreviewError::InvalidAsset);
        };

        let texture = instance.device().create_texture_with_data(
            instance.queue(),
            &texture_desc,
            TextureDataOrder::LayerMajor,
            frame.buffer(),
        );

        let texture_view = texture.create_view(&Default::default());

        let texture_sampler = instance.device().create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout =
            instance
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

        let bind_group = instance.device().create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        Ok(Self {
            bind_group,
            bind_group_layout,
        })
    }

    /// The bind group for this material texture.
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    /// The bind group layout for this material texture.
    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}
