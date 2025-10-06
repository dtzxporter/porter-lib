use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;
use porter_math::Vector3;
use porter_model::Skeleton;
use porter_utils::AsThisSlice;

/// A 3d render skeleton.
pub struct RenderSkeleton {
    vertex_buffer: Buffer,
    render_pipeline: RenderPipeline,
    pub(crate) bone_count: usize,
}

impl RenderSkeleton {
    /// Constructs  a new render skeleton from the given skeleton.
    pub fn from_skeleton(
        instance: &GPUInstance,
        bind_group_layouts: &[&BindGroupLayout],
        skeleton: &Skeleton,
    ) -> Self {
        let mut vertex_buffer = Vec::new();

        for bone in skeleton.bones.iter().rev() {
            vertex_buffer.push(bone.world_position);

            if bone.parent > -1 {
                vertex_buffer.push(skeleton.bones[bone.parent as usize].world_position);
            } else {
                vertex_buffer.push(Vector3::zero());
            }
        }

        let vertex_buffer = instance.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: vertex_buffer.as_slice().as_this_slice(),
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
                    entry_point: Some("vs_bone_main"),
                    buffers: &[VertexBufferLayout {
                        array_stride: size_of::<Vector3>() as BufferAddress,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        }],
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
                    depth_compare: CompareFunction::Always,
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
                    entry_point: Some("fs_bone_main"),
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

        Self {
            vertex_buffer,
            render_pipeline,
            bone_count: skeleton.bones.len(),
        }
    }

    /// Draws the skeleton using the given render pass.
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.bone_count as u32 * 2, 0..1);
    }
}
