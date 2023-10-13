use wgpu::util::*;
use wgpu::*;

use porter_gpu::GPUInstance;
use porter_math::Angles;
use porter_math::Matrix4x4;
use porter_math::Quaternion;
use porter_math::Vector3;
use porter_utils::AsByteSlice;

/// The axis which points up in 3d.
#[derive(Debug, Clone, Copy)]
pub enum PreviewCameraUpAxis {
    #[allow(unused)]
    Y,
    Z,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PreviewCameraUniform {
    target: Vector3,
    view_matrix: Matrix4x4,
    inverse_view_matrix: Matrix4x4,
    projection_matrix: Matrix4x4,
    model_matrix: Matrix4x4,
    inverse_model_matrix: Matrix4x4,
    default_shaded: u32,
}

/// A 3d preview camera.
#[derive(Debug)]
pub struct PreviewCamera {
    theta: f32,
    phi: f32,
    radius: f32,
    up: f32,
    uniforms: PreviewCameraUniform,
    uniform_buffer: Buffer,
    uniform_bind_group_layout: BindGroupLayout,
    uniform_bind_group: BindGroup,
    orthographic: Option<(f32, f32, f32)>,
}

impl PreviewCamera {
    /// Constructs a new preview camera instance.
    pub fn new(
        instance: &GPUInstance,
        theta: f32,
        phi: f32,
        radius: f32,
        up_axis: PreviewCameraUpAxis,
    ) -> Self {
        let model_matrix = match up_axis {
            PreviewCameraUpAxis::Y => Matrix4x4::new(),
            PreviewCameraUpAxis::Z => {
                Quaternion::from_euler_angles(-90.0, 0.0, 0.0, Angles::Degrees).matrix4x4()
            }
        };

        let uniforms = PreviewCameraUniform {
            target: Vector3::zero(),
            view_matrix: Matrix4x4::new(),
            inverse_view_matrix: Matrix4x4::new(),
            projection_matrix: Matrix4x4::new(),
            model_matrix,
            inverse_model_matrix: model_matrix.inverse(),
            default_shaded: 0,
        };

        let uniform_buffer = instance.device().create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: uniforms.as_byte_slice(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            instance
                .device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let uniform_bind_group = instance.device().create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            theta,
            phi,
            radius,
            up: 1.0,
            uniforms,
            uniform_buffer,
            uniform_bind_group_layout,
            uniform_bind_group,
            orthographic: None,
        }
    }

    /// Returns the uniform bind group.
    pub fn uniform_bind_group(&self) -> &BindGroup {
        &self.uniform_bind_group
    }

    /// Returns the uniform bind group layout.
    pub fn uniform_bind_group_layout(&self) -> &BindGroupLayout {
        &self.uniform_bind_group_layout
    }

    /// Returns true if in orthographic mode.
    pub fn is_orthographic(&self) -> bool {
        self.orthographic.is_some()
    }

    /// Sets whether or not the camera is in orthographic mode.
    pub fn set_orthographic(&mut self, orthographic: Option<(f32, f32, f32)>) {
        self.orthographic = orthographic;
    }

    /// Sets the orthographic scale value.
    pub fn set_orthographic_scale(&mut self, scale: f32) {
        if let Some((_, _, o_scale)) = &mut self.orthographic {
            *o_scale = scale;
        }
    }

    /// Toggles the default shaded camera view.
    pub fn toggle_shaded(&mut self) {
        self.uniforms.default_shaded = if self.uniforms.default_shaded == 1 {
            0
        } else {
            1
        };
    }

    /// Updates the current uniforms on the gpu.
    pub fn update(&mut self, instance: &GPUInstance, width: f32, height: f32) {
        if let Some((o_width, o_height, o_scale)) = self.orthographic {
            self.uniforms.projection_matrix =
                Matrix4x4::orthographic(0.0, width, height, 0.0, -1.0, 1.0);

            let center_x = (width - (o_width * o_scale)) / 2.0;
            let center_y = (height - (o_height * o_scale)) / 2.0;

            self.uniforms.view_matrix =
                Matrix4x4::create_position(Vector3::new(center_x, center_y, 0.0))
                    * Matrix4x4::create_scale(Vector3::new(o_scale, o_scale, 0.0));

            self.uniforms.inverse_view_matrix = self.uniforms.view_matrix.inverse();
            self.uniforms.inverse_model_matrix = self.uniforms.model_matrix.inverse();
        } else {
            self.uniforms.projection_matrix =
                Matrix4x4::perspective_fov(65.0, width / height, 0.1, 10000.0);
            self.uniforms.view_matrix = Matrix4x4::look_at(
                self.camera_position(),
                self.uniforms.target,
                Vector3::new(0.0, self.up, 0.0),
            );
            self.uniforms.inverse_view_matrix = self.uniforms.view_matrix.inverse();
            self.uniforms.inverse_model_matrix = self.uniforms.model_matrix.inverse();
        }

        instance
            .queue()
            .write_buffer(&self.uniform_buffer, 0, self.uniforms.as_byte_slice());
    }

    /// Resets the camera.
    pub fn reset(&mut self, theta: f32, phi: f32, radius: f32) {
        self.theta = theta;
        self.phi = phi;
        self.radius = radius;
        self.up = 1.0;
        self.uniforms.target = Vector3::zero();

        if self.radius <= 0.0 {
            self.radius = 30.0;

            let look = (self.uniforms.target - self.camera_position()).normalized();

            self.uniforms.target += look * 30.0;
        }
    }

    /// Rotates the camera by theta/phi.
    pub fn rotate(&mut self, theta: f32, phi: f32) {
        if self.up > 0.0 {
            self.theta += theta;
        } else {
            self.theta -= theta;
        }

        self.phi += phi;

        if self.phi > std::f32::consts::PI {
            self.phi -= std::f32::consts::TAU;
        } else if self.phi < -std::f32::consts::TAU {
            self.phi += std::f32::consts::TAU;
        }

        if (self.phi > 0.0 && self.phi < std::f32::consts::PI)
            || (self.phi < -std::f32::consts::PI && self.phi > -std::f32::consts::TAU)
        {
            self.up = 1.0;
        } else {
            self.up = -1.0;
        }
    }

    /// Zooms the camera by the given distance.
    pub fn zoom(&mut self, distance: f32) {
        self.radius -= distance;

        if self.radius <= 0.0 {
            self.radius = 30.0;

            let look = (self.uniforms.target - self.camera_position()).normalized();

            self.uniforms.target += look * 30.0;
        }
    }

    /// Pans the camera around the current z axis.
    pub fn pan(&mut self, x: f32, y: f32) {
        let look = (self.uniforms.target - self.camera_position()).normalized();
        let world_up = Vector3::new(0.0, self.up, 0.0);

        let right = look.cross(world_up);
        let up = look.cross(right);

        self.uniforms.target += (right * x) + (up * y);
    }

    /// Returns the camera position.
    fn camera_position(&self) -> Vector3 {
        self.uniforms.target + self.to_cartesian()
    }

    /// Returns the camera radius as cartesian units.
    fn to_cartesian(&self) -> Vector3 {
        Vector3::new(
            self.radius * self.phi.sin() * self.theta.sin(),
            self.radius * self.phi.cos(),
            self.radius * self.phi.sin() * self.theta.cos(),
        )
    }
}
