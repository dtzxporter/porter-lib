use wgpu::util::*;
use wgpu::*;

use porter_model::MaterialTextureRefUsage;
use porter_model::Model;

use porter_gpu::GPUInstance;
use porter_gpu::gpu_instance;

use porter_math::Angles;
use porter_math::Axis;
use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector2;
use porter_math::Vector3;

use porter_utils::AsAligned;
use porter_utils::AsThisSlice;

use porter_texture::Image;
use porter_texture::TextureExtensions;

use crate::PreviewCamera;
use crate::PreviewError;
use crate::PreviewKeyState;
use crate::RenderImage;
use crate::RenderMaterial;
use crate::RenderModel;
use crate::RenderType;

/// Renders 'preview' versions of models, animations, images, and materials.
pub struct PreviewRenderer {
    instance: &'static GPUInstance,
    wireframe: bool,
    show_bones: bool,
    show_grid: bool,
    width: f32,
    height: f32,
    far_clip: f32,
    output_texture: Texture,
    output_texture_view: TextureView,
    output_buffer: Buffer,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    msaa_texture: Texture,
    msaa_texture_view: TextureView,
    grid_size: u32,
    grid_render_buffer: Buffer,
    grid_render_pipeline: RenderPipeline,
    render: Option<RenderType>,
    render_name: Option<String>,
    camera: PreviewCamera,
    scale: u32,
}

/// The minimum preview size.
const MIN_SIZE: u32 = 256;

/// The size of the grid.
const GRID_SIZE: f32 = 120.0;
/// The size of each subdivision.
const GRID_STEP: f32 = 2.0;

/// Utility to create the output texture.
fn create_output_texture(instance: &GPUInstance, width: u32, height: u32) -> Texture {
    instance.device().create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Utility to create the depth texture.
fn create_depth_texture(instance: &GPUInstance, width: u32, height: u32) -> Texture {
    instance.device().create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Utility to create the MSAA texture.
fn create_msaa_texture(instance: &GPUInstance, width: u32, height: u32) -> Texture {
    instance.device().create_texture(&TextureDescriptor {
        label: None,
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 4,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

/// Utility to create the output texture buffer.
fn create_output_buffer(instance: &GPUInstance, width: u32, height: u32) -> Buffer {
    let output_format = TextureFormat::Rgba8Unorm;

    instance.device().create_buffer(&BufferDescriptor {
        label: None,
        size: output_format.buffer_size_aligned(width, height),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

/// Utility to create the grid render resources.
fn create_grid_render(
    instance: &GPUInstance,
    bind_group_layouts: &[&BindGroupLayout],
) -> (u32, Buffer, RenderPipeline) {
    let size = GRID_SIZE;
    let min_size = -size;
    let step = GRID_STEP;

    let mut buffer = Vec::new();
    let mut i = min_size;

    while i <= size {
        let color = if i == 0.0 {
            Vector3::new(0.153, 0.608, 0.831)
        } else {
            Vector3::new(0.70, 0.70, 0.70)
        };

        buffer.push(Vector3::new(i, 0.0, size));
        buffer.push(color);
        buffer.push(Vector3::new(i, 0.0, min_size));
        buffer.push(color);
        buffer.push(Vector3::new(size, 0.0, i));
        buffer.push(color);
        buffer.push(Vector3::new(min_size, 0.0, i));
        buffer.push(color);

        i += step;
    }

    let size = (buffer.len() / 2) as u32;

    let buffer = instance.device().create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: buffer.as_slice().as_this_slice(),
        usage: BufferUsages::VERTEX,
    });

    let render_pipeline_layout =
        instance
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts,
                push_constant_ranges: &[],
            });

    let render_pipeline = instance
        .device()
        .create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: instance.gpu_preview_shader(),
                entry_point: Some("vs_grid_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: (size_of::<Vector3>() * 2) as BufferAddress,
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
                            format: VertexFormat::Float32x3,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Line,
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
                entry_point: Some("fs_grid_main"),
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

    (size, buffer, render_pipeline)
}

impl PreviewRenderer {
    /// Constructs a new instance of the preview renderer.
    pub fn new() -> Self {
        let instance = gpu_instance();
        let output_texture = create_output_texture(instance, MIN_SIZE, MIN_SIZE);
        let output_buffer = create_output_buffer(instance, MIN_SIZE, MIN_SIZE);
        let depth_texture = create_depth_texture(instance, MIN_SIZE, MIN_SIZE);
        let msaa_texture = create_msaa_texture(instance, MIN_SIZE, MIN_SIZE);

        let camera = PreviewCamera::new(
            instance,
            0.5 * std::f32::consts::PI,
            0.45 * std::f32::consts::PI,
            100.0,
        );

        let (grid_size, grid_render_buffer, grid_render_pipeline) =
            create_grid_render(instance, &[camera.uniform_bind_group_layout()]);

        Self {
            instance,
            wireframe: false,
            show_bones: true,
            show_grid: true,
            width: MIN_SIZE as f32,
            height: MIN_SIZE as f32,
            far_clip: 10000.0,
            output_texture_view: output_texture.create_view(&Default::default()),
            output_texture,
            output_buffer,
            depth_texture_view: depth_texture.create_view(&Default::default()),
            depth_texture,
            msaa_texture_view: msaa_texture.create_view(&Default::default()),
            msaa_texture,
            grid_size,
            grid_render_buffer,
            grid_render_pipeline,
            render: None,
            render_name: None,
            camera,
            scale: 100,
        }
    }

    /// Sets the image asset to preview.
    pub fn set_preview_image(&mut self, name: String, image: Image) -> Result<(), PreviewError> {
        let render_image = RenderImage::from_image(
            self.instance,
            &[self.camera.uniform_bind_group_layout()],
            &image,
        )?;

        let scale = (self.width / image.width() as f32).min(self.height / image.height() as f32);

        self.scale = 100.min((scale * 100.0) as u32);

        self.camera.set_orthographic(Some((
            image.width() as f32,
            image.height() as f32,
            self.scale as f32 / 100.0,
        )));

        self.render = Some(RenderType::Image(render_image));
        self.render_name = Some(name);

        self.update_camera();

        Ok(())
    }

    /// Sets the material asset to preview.
    pub fn set_preview_material(
        &mut self,
        name: String,
        material: Vec<(MaterialTextureRefUsage, Image)>,
    ) -> Result<(), PreviewError> {
        let render_material = RenderMaterial::from_images(
            self.instance,
            &[self.camera.uniform_bind_group_layout()],
            &material,
        )?;

        let scale = (self.width / render_material.width() as f32)
            .min(self.height / render_material.height() as f32);

        self.scale = 100.min((scale * 100.0) as u32);

        self.camera.set_orthographic(Some((
            render_material.width() as f32,
            render_material.height() as f32,
            self.scale as f32 / 100.0,
        )));

        self.render = Some(RenderType::Material(render_material));
        self.render_name = Some(name);

        self.update_camera();

        Ok(())
    }

    /// Sets the model asset to preview.
    pub fn set_preview_model(
        &mut self,
        name: String,
        model: Model,
        materials: Vec<Option<Image>>,
        srgb: bool,
    ) -> Result<(), PreviewError> {
        let render_model = RenderModel::from_model(
            self.instance,
            &[self.camera.uniform_bind_group_layout()],
            &model,
            &materials,
            srgb,
        )?;

        let model_matrix = match model.up_axis {
            Axis::X => {
                Quaternion::from_euler(Vector3::new(0.0, 0.0, -90.0), Angles::Degrees).to_4x4()
            }
            Axis::Y => Matrix4x4::new(),
            Axis::Z => {
                Quaternion::from_euler(Vector3::new(-90.0, 0.0, 0.0), Angles::Degrees).to_4x4()
            }
        };

        self.camera.set_orthographic(None);
        self.camera.set_model_matrix(model_matrix);

        self.render = Some(RenderType::Model(render_model));
        self.render_name = Some(name);

        self.update_camera();

        Ok(())
    }

    /// Clears the asset being previewed.
    pub fn clear_preview(&mut self) {
        self.render = None;
        self.render_name = None;

        self.camera.set_orthographic(None);
        self.update_camera();
    }

    /// Returns true if the preview is empty.
    pub fn is_empty_preview(&self) -> bool {
        self.render.is_none()
    }

    /// Resizes the renderer output.
    pub fn resize(&mut self, width: f32, height: f32, far_clip: f32) {
        let width = width.max(1.0);
        let height = height.max(1.0);

        if self.width == width && self.height == height && self.far_clip == far_clip {
            return;
        }

        self.width = width;
        self.height = height;
        self.far_clip = far_clip;

        self.output_texture =
            create_output_texture(self.instance, self.width as u32, self.height as u32);
        self.output_texture_view = self.output_texture.create_view(&Default::default());
        self.output_buffer =
            create_output_buffer(self.instance, self.width as u32, self.height as u32);

        self.depth_texture =
            create_depth_texture(self.instance, self.width as u32, self.height as u32);
        self.depth_texture_view = self.depth_texture.create_view(&Default::default());

        self.msaa_texture =
            create_msaa_texture(self.instance, self.width as u32, self.height as u32);
        self.msaa_texture_view = self.msaa_texture.create_view(&Default::default());

        self.update_camera();
    }

    /// Cycles to the next material in the list.
    pub fn cycle_material(&mut self) {
        if let Some(RenderType::Material(material)) = &mut self.render {
            material.next();

            let scale =
                (self.width / material.width() as f32).min(self.height / material.height() as f32);

            self.scale = 100.min((scale * 100.0) as u32);

            self.camera.set_orthographic(Some((
                material.width() as f32,
                material.height() as f32,
                self.scale as f32 / 100.0,
            )));

            self.update_camera();
        }
    }

    /// Toggles the wireframe view.
    pub fn toggle_wireframe(&mut self) {
        self.wireframe = !self.wireframe;
    }

    /// Toggles the bone view.
    pub fn toggle_bones(&mut self) {
        self.show_bones = !self.show_bones;
    }

    /// Toggles the grid view.
    pub fn toggle_grid(&mut self) {
        self.show_grid = !self.show_grid;
    }

    /// Toggles the shaded view.
    pub fn toggle_shaded(&mut self) {
        self.camera.toggle_shaded();
        self.update_camera();
    }

    /// Performs a reset operation.
    pub fn reset_view(&mut self) {
        if !self.camera.is_orthographic() {
            self.camera.reset(
                0.5 * std::f32::consts::PI,
                0.45 * std::f32::consts::PI,
                100.0,
            );

            self.update_camera();
        }
    }

    /// Performs a scrolling operation.
    pub fn scroll_delta(&mut self, delta: f32) {
        if self.camera.is_orthographic() {
            if delta > 0.0 {
                self.scale = self.scale.wrapping_add(3);
            } else {
                self.scale = self.scale.wrapping_sub(3);
            }

            self.scale = (self.scale as i32).clamp(0, 200) as u32;

            self.camera
                .set_orthographic_scale(self.scale as f32 / 100.0);
        } else {
            self.camera.zoom(delta * 0.5);
        }

        self.update_camera();
    }

    /// Performs a mouse move operation.
    pub fn mouse_move<D: Into<Vector2>>(&mut self, delta: D, key_state: PreviewKeyState) {
        let delta = delta.into();

        if key_state.maya && !key_state.alt {
            return;
        }

        let mut dirty = false;

        if key_state.maya {
            if key_state.left {
                let phi = delta.y / 200.0;
                let theta = delta.x / 200.0;

                self.camera.rotate(theta, phi);

                dirty = true;
            } else if key_state.right {
                self.camera.zoom(-(delta.x / 2.0));

                dirty = true;
            } else if key_state.middle {
                let x = delta.x * 0.1;
                let y = delta.y * 0.1;

                self.camera.pan(x, y);

                dirty = true;
            }
        } else if key_state.middle && key_state.shift {
            let x = delta.x * 0.1;
            let y = delta.y * 0.1;

            self.camera.pan(x, y);

            dirty = true;
        } else if key_state.middle && key_state.alt {
            self.camera.zoom(-(delta.x / 2.0));

            dirty = true;
        } else if key_state.middle {
            let phi = delta.y / 200.0;
            let theta = delta.x / 200.0;

            self.camera.rotate(theta, phi);

            dirty = true;
        }

        if dirty {
            self.update_camera();
        }
    }

    /// Returns the statistics for the current render assset.
    pub fn statistics(&self) -> Vec<(String, String)> {
        match &self.render {
            Some(RenderType::Model(model)) => {
                vec![
                    (
                        String::from("Name"),
                        self.render_name
                            .clone()
                            .unwrap_or_else(|| String::from("N/A")),
                    ),
                    (String::from("Meshes"), model.mesh_count().to_string()),
                    (String::from("Verts"), model.vertex_count().to_string()),
                    (String::from("Tris"), model.face_count().to_string()),
                    (String::from("Bones"), model.bone_count().to_string()),
                ]
            }
            Some(RenderType::Image(image)) => {
                vec![
                    (
                        String::from("Name"),
                        self.render_name
                            .clone()
                            .unwrap_or_else(|| String::from("N/A")),
                    ),
                    (String::from("Width"), image.width().to_string()),
                    (String::from("Height"), image.height().to_string()),
                    (String::from("Scale"), format!("{}%", self.scale)),
                ]
            }
            Some(RenderType::Material(material)) => {
                let mut result = vec![
                    (
                        String::from("Name"),
                        self.render_name
                            .clone()
                            .unwrap_or_else(|| String::from("N/A")),
                    ),
                    (
                        String::from("Image"),
                        if material.is_empty() {
                            String::from("0 of 0")
                        } else {
                            format!("{} of {}", material.index() + 1, material.len())
                        },
                    ),
                ];

                if material.is_error() {
                    result.push((
                        String::from("Status"),
                        String::from("Unable to preview this image"),
                    ));
                } else {
                    result.extend([
                        (String::from("Usage"), material.usage()),
                        (String::from("Width"), material.width().to_string()),
                        (String::from("Height"), material.height().to_string()),
                        (String::from("Scale"), format!("{}%", self.scale)),
                    ]);
                }

                result
            }
            _ => vec![(String::from("Name"), String::from("N/A"))],
        }
    }

    // Get the rendered output.
    pub fn render(&self) -> (u32, u32, Vec<u8>) {
        let mut encoder = self
            .instance
            .device()
            .create_command_encoder(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.msaa_texture_view,
                resolve_target: Some(&self.output_texture_view),
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.066,
                        g: 0.066,
                        b: 0.066,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            occlusion_query_set: None,
            timestamp_writes: None,
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_bind_group(0, self.camera.uniform_bind_group(), &[]);

        let mut draw_grid = || {
            if self.show_grid {
                render_pass.set_pipeline(&self.grid_render_pipeline);
                render_pass.set_vertex_buffer(0, self.grid_render_buffer.slice(..));
                render_pass.draw(0..self.grid_size, 0..1);
            }
        };

        match &self.render {
            Some(RenderType::Model(model)) => {
                draw_grid();

                model.draw(&mut render_pass, self.show_bones, self.wireframe);
            }
            Some(RenderType::Image(image)) => {
                image.draw(&mut render_pass);
            }
            Some(RenderType::Material(material)) => {
                material.draw(&mut render_pass);
            }
            _ => draw_grid(),
        }

        drop(render_pass);

        let output_format = TextureFormat::Rgba8Unorm;
        let block_dimensions = output_format.block_dimensions();
        let bytes_per_row = output_format.bytes_per_row(self.width as u32);

        {
            encoder.copy_texture_to_buffer(
                TexelCopyTextureInfo {
                    texture: &self.output_texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                TexelCopyBufferInfo {
                    buffer: &self.output_buffer,
                    layout: TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bytes_per_row.as_aligned(COPY_BYTES_PER_ROW_ALIGNMENT)),
                        rows_per_image: None,
                    },
                },
                Extent3d {
                    width: self.width as u32,
                    height: self.height as u32,
                    depth_or_array_layers: 1,
                },
            )
        }

        let submission = self.instance.queue().submit(Some(encoder.finish()));

        let output_slice = self.output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);

        output_slice.map_async(MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        let _ = self
            .instance
            .device()
            .poll(MaintainBase::WaitForSubmissionIndex(submission));

        if rx.recv().unwrap().is_err() {
            return (0, 0, Vec::new());
        }

        let buffer = output_slice.get_mapped_range();

        let nbh = (self.height as usize).div_ceil(block_dimensions.1 as usize);

        let truncated_size = bytes_per_row as usize * nbh;
        let aligned_bytes_per_row = bytes_per_row.as_aligned(COPY_BYTES_PER_ROW_ALIGNMENT) as usize;

        let pixels = if buffer.len() == truncated_size {
            buffer.to_vec()
        } else {
            let mut result = vec![0; truncated_size];

            for (i, row) in result.chunks_exact_mut(bytes_per_row as usize).enumerate() {
                let source = i * aligned_bytes_per_row;

                row.copy_from_slice(&buffer[source..source + bytes_per_row as usize]);
            }

            result
        };

        drop(buffer);

        self.output_buffer.unmap();

        (self.width as u32, self.height as u32, pixels)
    }

    /// Updates the camera with current parameters.
    fn update_camera(&mut self) {
        let srgb = match &self.render {
            Some(RenderType::Image(image)) => image.srgb(),
            Some(RenderType::Material(material)) => material.srgb(),
            Some(RenderType::Model(model)) => model.srgb(),
            None => false,
        };

        self.camera
            .update(self.instance, self.width, self.height, srgb, self.far_clip);
    }
}

impl Default for PreviewRenderer {
    fn default() -> Self {
        Self::new()
    }
}
