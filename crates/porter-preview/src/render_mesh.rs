use std::sync::Arc;

use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;
use porter_math::Vector2;
use porter_math::Vector3;
use porter_model::Mesh;
use porter_utils::AsThisSlice;

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
    ) -> Self {
        let stride = (std::mem::size_of::<Vector3>() * 2) + std::mem::size_of::<Vector2>();
        let slice = mesh.vertices.as_slice();

        let material_texture = match mesh.materials.first() {
            Some(-1) => material_textures[material_textures.len() - 1].clone(),
            Some(index) => material_textures[*index as usize].clone(),
            None => material_textures[material_textures.len() - 1].clone(),
        };

        let mut vertex_buffer = vec![0; stride * mesh.vertices.len()];
        let mut offset = 0;

        for chunk in vertex_buffer.chunks_exact_mut(stride) {
            chunk.copy_from_slice(&slice[offset..offset + stride]);
            offset += mesh.vertices.stride();
        }

        let vertex_buffer = instance
            .device()
            .create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: &vertex_buffer,
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
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: stride as BufferAddress,
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
                            format: VertexFormat::Float32x3,
                        },
                        VertexAttribute {
                            offset: (std::mem::size_of::<Vector3>() * 2) as BufferAddress,
                            shader_location: 2,
                            format: VertexFormat::Float32x2,
                        },
                    ],
                }],
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
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba8Unorm,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
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

        Self {
            render_pipeline,
            render_pipeline_wireframe,
            vertex_buffer,
            vertex_count: mesh.vertices.len(),
            face_buffer,
            face_count: mesh.faces.len(),
            material_texture,
        }
    }

    /// Draws the mesh using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, wireframe: bool) {
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
