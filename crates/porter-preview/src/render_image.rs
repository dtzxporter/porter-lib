use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;
use porter_math::Vector2;
use porter_math::Vector3;
use porter_texture::Image;
use porter_utils::AsByteSlice;
use porter_utils::AsThisSlice;

/// A 3d render image.
pub struct RenderImage {
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    width: u32,
    height: u32,
}

impl RenderImage {
    /// Constructs a new render image from the given image.
    pub fn from_image(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        image: &Image,
    ) -> Self {
        let format_convert = image.format().to_wgpu();
        let format = *format_convert
            .as_ref()
            .unwrap_or(&TextureFormat::Rgba8Unorm);

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

        let texture = if let (Some(frame), Ok(_)) = (image.frames().first(), format_convert) {
            instance.device().create_texture_with_data(
                instance.queue(),
                &texture_desc,
                TextureDataOrder::LayerMajor,
                frame.buffer(),
            )
        } else {
            instance.device().create_texture_with_data(
                instance.queue(),
                &texture_desc,
                TextureDataOrder::LayerMajor,
                &vec![0; image.width() as usize * image.height() as usize * 0x4],
            )
        };

        let texture_view = texture.create_view(&Default::default());

        let texture_sampler = instance.device().create_sampler(&SamplerDescriptor {
            mag_filter: FilterMode::Linear,
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
                    entry_point: "vs_image_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: (std::mem::size_of::<Vector3>()
                            + std::mem::size_of::<Vector2>())
                            as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: VertexFormat::Float32x3,
                            },
                            VertexAttribute {
                                offset: std::mem::size_of::<Vector3>() as BufferAddress,
                                shader_location: 1,
                                format: VertexFormat::Float32x2,
                            },
                        ],
                    }],
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
                    entry_point: "fs_image_main",
                    targets: &[Some(ColorTargetState {
                        format: TextureFormat::Rgba8Unorm,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let mut vertex_buffer = Vec::new();

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

        Self {
            bind_group,
            render_pipeline,
            vertex_buffer,
            width: image.width(),
            height: image.height(),
        }
    }

    /// Returns the width of the image.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Draws the image using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(1, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
