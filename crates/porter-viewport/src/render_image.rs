use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;

use porter_math::Vector2;
use porter_math::Vector3;

use porter_texture::Image;
use porter_texture::ImageFormat;

use porter_utils::AsByteSlice;
use porter_utils::AsThisSlice;
use porter_utils::VecExt;

use crate::PreviewError;

/// A 3d render image.
pub struct RenderImage {
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    width: u32,
    height: u32,
    format: ImageFormat,
}

impl RenderImage {
    /// Constructs a new render image from the given image.
    pub fn from_image(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        image: &Image,
    ) -> Result<Self, PreviewError> {
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

        let render_pipeline_layout =
            instance
                .device()
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[bind_group_layouts, &[&bind_group_layout]].concat(),
                    push_constant_ranges: &[],
                });

        let render_pipeline = instance
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&render_pipeline_layout),
                vertex: VertexState {
                    module: instance.gpu_preview_shader(),
                    entry_point: Some("vs_image_main"),
                    buffers: &[VertexBufferLayout {
                        array_stride: (size_of::<Vector3>() + size_of::<Vector2>())
                            as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float32x3,
                            },
                            VertexAttribute {
                                offset: size_of::<Vector3>() as BufferAddress,
                                shader_location: 1,
                                format: VertexFormat::Float32x2,
                            },
                        ],
                    }],
                    compilation_options: Default::default(),
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    module: instance.gpu_preview_shader(),
                    entry_point: if format.components() > 1 {
                        Some("fs_image_main")
                    } else {
                        Some("fs_image_grayscale")
                    },
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview: None,
                cache: None,
            });

        let mut vertex_buffer =
            Vec::try_with_exact_capacity((size_of::<Vector3>() * 6) + (size_of::<Vector2>() * 6))?;

        let width = image.width() as f32;
        let height = image.height() as f32;

        vertex_buffer.extend_from_slice(Vector3::new(-1.0, -1.0, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector2::new(0.0, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector3::new(width, -1.0, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector2::new(1.0, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector3::new(width, height, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector2::new(1.0, 1.0).as_byte_slice());

        vertex_buffer.extend_from_slice(Vector3::new(-1.0, -1.0, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector2::new(0.0, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector3::new(width, height, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector2::new(1.0, 1.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector3::new(-1.0, height, 0.0).as_byte_slice());
        vertex_buffer.extend_from_slice(Vector2::new(0.0, 1.0).as_byte_slice());

        let vertex_buffer = instance.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertex_buffer.as_slice().as_this_slice(),
            usage: BufferUsages::VERTEX,
        });

        Ok(Self {
            bind_group,
            render_pipeline,
            vertex_buffer,
            width: image.width(),
            height: image.height(),
            format: image.format(),
        })
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns whether or not the image is in sRGB colorspace.
    pub fn srgb(&self) -> bool {
        self.format.is_srgb()
    }

    /// Draws the image using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
