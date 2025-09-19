use std::io::Cursor;
use std::sync::Arc;

use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;

use porter_math::Vector2;
use porter_math::Vector3;

use porter_model::Mesh;

use porter_utils::AsThisSlice;
use porter_utils::StructWriteExt;
use porter_utils::VecExt;

use crate::PreviewError;
use crate::RenderMaterialTexture;

/// A 3d render mesh.
pub struct RenderMesh {
    render_pipeline: RenderPipeline,
    render_pipeline_wireframe: RenderPipeline,
    vertex_buffer: Buffer,
    pub(crate) vertex_count: usize,
    face_buffer: Buffer,
    pub(crate) face_count: usize,
    material_texture: Arc<RenderMaterialTexture>,
}

impl RenderMesh {
    /// Constructs a new render mesh from the given mesh.
    pub fn from_mesh(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        mesh: &Mesh,
        material_textures: &[Arc<RenderMaterialTexture>],
        culling: bool,
    ) -> Result<Self, PreviewError> {
        let vertex_stride = (size_of::<Vector3>() * 2) + size_of::<Vector2>();

        let material_texture = match mesh.material {
            Some(index) => material_textures
                .get(index)
                .cloned()
                .unwrap_or_else(|| material_textures[material_textures.len() - 1].clone()),
            None => material_textures[material_textures.len() - 1].clone(),
        };

        let vertex_buffer: Vec<u8> =
            Vec::try_with_exact_capacity(vertex_stride * mesh.vertices.len())?;

        let mut vertex_buffer = Cursor::new(vertex_buffer);

        for v in 0..mesh.vertices.len() {
            let vertex = mesh.vertices.vertex(v);

            vertex_buffer.write_struct(vertex.position())?;
            vertex_buffer.write_struct(vertex.normal())?;

            if mesh.vertices.uv_layers() > 0 {
                vertex_buffer.write_struct(vertex.uv(0))?;
            } else {
                vertex_buffer.write_struct(Vector2::zero())?;
            }
        }

        let vertex_buffer = instance
            .device()
            .create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: &vertex_buffer.into_inner(),
                usage: BufferUsages::VERTEX,
            });

        let face_buffer = instance
            .device()
            .create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: mesh.faces.as_slice().as_this_slice(),
                usage: BufferUsages::INDEX,
            });

        let render_pipeline_layout =
            instance
                .device()
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        bind_group_layouts,
                        &[material_texture.bind_group_layout()],
                    ]
                    .concat(),
                    push_constant_ranges: &[],
                });

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: instance.gpu_preview_shader(),
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: vertex_stride as BufferAddress,
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
                        VertexAttribute {
                            offset: (size_of::<Vector3>() * 2) as BufferAddress,
                            shader_location: 2,
                            format: VertexFormat::Float32x2,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: if culling { Some(Face::Back) } else { None },
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
                entry_point: if culling {
                    Some("fs_main")
                } else {
                    Some("fs_main_nocull")
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
        };

        let render_pipeline = instance
            .device()
            .create_render_pipeline(&render_pipeline_desc);

        let render_pipeline_wireframe =
            instance
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    primitive: PrimitiveState {
                        polygon_mode: PolygonMode::Line,
                        ..render_pipeline_desc.primitive
                    },
                    ..render_pipeline_desc
                });

        Ok(Self {
            render_pipeline,
            render_pipeline_wireframe,
            vertex_buffer,
            vertex_count: mesh.vertices.len(),
            face_buffer,
            face_count: mesh.faces.len(),
            material_texture,
        })
    }

    /// Draws the mesh using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, wireframe: bool) {
        if self.vertex_count == 0 || self.face_count == 0 {
            return;
        }

        if wireframe {
            render_pass.set_pipeline(&self.render_pipeline_wireframe);
        } else {
            render_pass.set_pipeline(&self.render_pipeline);
        }

        render_pass.set_bind_group(1, self.material_texture.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.face_buffer.slice(..), IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.face_count as u32 * 3, 0, 0..1);
    }
}
