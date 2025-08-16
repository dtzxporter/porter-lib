use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::*;

use std::sync::mpsc;

use porter_utils::AsAligned;
use porter_utils::AsByteSlice;

use porter_gpu::gpu_instance;
use porter_gpu::GPUInstance;

use crate::ImageConvertOptions;
use crate::TextureError;
use crate::TextureExtensions;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
struct GPUOptionsUniform {
    input_unorm: u32,
    output_unorm: u32,
    invert_y: u32,
}

/// Converts textures from one format to another (uncompressed only).
pub struct GPUConverter {
    width: u32,
    height: u32,
    input_format: TextureFormat,
    output_format: TextureFormat,
    options: ImageConvertOptions,
    instance: &'static GPUInstance,
}

impl GPUConverter {
    /// Creates a new instance of the GPU converter.
    pub fn new(
        width: u32,
        height: u32,
        input_format: TextureFormat,
        output_format: TextureFormat,
    ) -> Self {
        Self {
            width,
            height,
            input_format,
            output_format,
            options: Default::default(),
            instance: gpu_instance(),
        }
    }

    /// Sets conversion options.
    pub fn set_options(&mut self, options: ImageConvertOptions) {
        self.options = options;
    }

    /// The texture size of the input and output textures.
    #[inline(always)]
    fn texture_size(&self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    /// Creates the input texture data layout.
    #[inline(always)]
    fn input_texture_data_layout(&self) -> ImageDataLayout {
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(self.input_format.bytes_per_row(self.width)),
            rows_per_image: None,
        }
    }

    /// Creates the input options buffer for the converter.
    fn create_input_options(&self) -> Buffer {
        let uniforms = GPUOptionsUniform {
            input_unorm: self.input_format.is_unorm() as u32,
            output_unorm: self.output_format.is_unorm() as u32,
            invert_y: (matches!(self.options, ImageConvertOptions::ReconstructZInvertY)
                || matches!(self.options, ImageConvertOptions::AutoReconstructZInvertY))
                as u32,
        };

        self.instance
            .device()
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: uniforms.as_byte_slice(),
                usage: BufferUsages::UNIFORM,
            })
    }

    /// Creates an input texture that matches our input format and texture size.
    fn create_input_texture(&self) -> Texture {
        self.instance.device().create_texture(&TextureDescriptor {
            label: None,
            size: self.texture_size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.input_format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        })
    }

    /// Creates an input sampler.
    fn create_input_sampler(&self) -> Sampler {
        self.instance.device().create_sampler(&Default::default())
    }

    /// Creates a bind group laypout for the fragment shader.
    fn create_bind_group_layout(&self) -> BindGroupLayout {
        self.instance
            .device()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            })
    }

    /// Creates a bind group for the input texture and texture sampler.
    fn create_bind_group(
        &self,
        bind_group_layout: &BindGroupLayout,
        input_options: &Buffer,
        input_texture_view: &TextureView,
        input_texture_sampler: &Sampler,
    ) -> BindGroup {
        self.instance
            .device()
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: input_options.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(input_texture_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(input_texture_sampler),
                    },
                ],
            })
    }

    /// Creates a render pipeline that will take the input and render to the target output.
    fn create_render_pipeline(&self, bind_group_layout: &BindGroupLayout) -> RenderPipeline {
        let pipeline_layout =
            self.instance
                .device()
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[bind_group_layout],
                    push_constant_ranges: &[],
                });

        let fragment_entry = match self.options {
            ImageConvertOptions::None => "fs_main",
            ImageConvertOptions::ReconstructZ | ImageConvertOptions::ReconstructZInvertY => {
                "fs_rz_main"
            }
            ImageConvertOptions::AutoReconstructZ
            | ImageConvertOptions::AutoReconstructZInvertY => {
                if matches!(self.input_format, TextureFormat::Bc5RgUnorm) {
                    "fs_rz_main"
                } else {
                    "fs_main"
                }
            }
            ImageConvertOptions::UniformScaleBias(_, _) => {
                unimplemented!()
            }
        };

        self.instance
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: self.instance.gpu_converter_shader(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Cw,
                    cull_mode: Some(Face::Back),
                    unclipped_depth: false,
                    polygon_mode: PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentState {
                    module: self.instance.gpu_converter_shader(),
                    entry_point: fragment_entry,
                    targets: &[Some(ColorTargetState {
                        format: self.output_format,
                        blend: Some(BlendState::REPLACE),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            })
    }

    /// Creates an output texture that matches our output format and size.
    fn create_output_texture(&self) -> Texture {
        self.instance.device().create_texture(&TextureDescriptor {
            label: None,
            size: self.texture_size(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.output_format,
            usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }

    /// Creates an output buffer based on the size of the output texture.
    fn create_output_buffer(&self) -> Buffer {
        self.instance.device().create_buffer(&BufferDescriptor {
            label: None,
            size: self
                .output_format
                .buffer_size_aligned(self.width, self.height),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    /// Sets up a render pass that renders the input texture to the output texture.
    fn begin_render_pass(
        &self,
        encoder: &mut CommandEncoder,
        output_texture_view: &TextureView,
        render_pipeline: &RenderPipeline,
        bind_group: &BindGroup,
    ) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output_texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            occlusion_query_set: None,
            timestamp_writes: None,
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    /// Creates a stage that copys the output texture to a output buffer.
    fn copy_texture_to_buffer(
        &self,
        encoder: &mut CommandEncoder,
        output_texture: &Texture,
        output_buffer: &Buffer,
    ) {
        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: output_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: output_buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        self.output_format
                            .bytes_per_row(self.width)
                            .as_aligned(COPY_BYTES_PER_ROW_ALIGNMENT),
                    ),
                    rows_per_image: None,
                },
            },
            self.texture_size(),
        )
    }

    /// Upload the CPU texture data directly to the GPU texture.
    fn upload_cpu_texture_gpu<I: AsRef<[u8]>>(&self, input: I, input_texture: &Texture) {
        self.instance.queue().write_texture(
            ImageCopyTexture {
                texture: input_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            input.as_ref(),
            self.input_texture_data_layout(),
            self.texture_size(),
        );
    }

    /// Downloads the GPU texture data to the CPU texture buffer.
    fn download_gpu_texture_cpu<O: AsMut<[u8]>>(
        &self,
        submission: SubmissionIndex,
        mut output: O,
        output_buffer: &Buffer,
    ) -> Result<(), TextureError> {
        let output_slice = output_buffer.slice(..);
        let (tx, rx) = mpsc::sync_channel(1);

        output_slice.map_async(MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.instance
            .device()
            .poll(MaintainBase::WaitForSubmissionIndex(submission));

        if rx.recv().unwrap().is_err() {
            return Err(TextureError::ConversionError);
        }

        let output = output.as_mut();
        let output_len = output.len();

        output
            .as_mut()
            .copy_from_slice(&output_slice.get_mapped_range()[..output_len]);

        Ok(())
    }

    /// Converts the texture data in input to the specified format in output.
    pub fn convert<I: AsRef<[u8]>, O: AsMut<[u8]>>(
        &self,
        input: I,
        output: O,
    ) -> Result<(), TextureError> {
        let input_texture = self.create_input_texture();

        self.upload_cpu_texture_gpu(input, &input_texture);

        let input_options = self.create_input_options();
        let input_texture_view = input_texture.create_view(&Default::default());
        let input_texture_sampler = self.create_input_sampler();

        let bind_group_layout = self.create_bind_group_layout();
        let bind_group = self.create_bind_group(
            &bind_group_layout,
            &input_options,
            &input_texture_view,
            &input_texture_sampler,
        );

        let render_pipeline = self.create_render_pipeline(&bind_group_layout);

        let output_texture = self.create_output_texture();

        let output_texture_view = output_texture.create_view(&Default::default());
        let output_buffer = self.create_output_buffer();

        let mut encoder = self
            .instance
            .device()
            .create_command_encoder(&Default::default());

        self.begin_render_pass(
            &mut encoder,
            &output_texture_view,
            &render_pipeline,
            &bind_group,
        );

        self.copy_texture_to_buffer(&mut encoder, &output_texture, &output_buffer);

        let submission = self.instance.queue().submit(Some(encoder.finish()));

        self.download_gpu_texture_cpu(submission, output, &output_buffer)
    }
}
