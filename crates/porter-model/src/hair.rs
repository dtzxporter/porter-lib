use porter_math::Matrix4x4;
use porter_math::Vector3;

use crate::Face;
use crate::FaceBuffer;
use crate::Mesh;
use crate::VertexBuffer;

/// A 3d hair definition.
#[derive(Debug, Clone, Default)]
pub struct Hair {
    /// Name of this hair, used for identification.
    pub name: Option<String>,
    /// Number of segments in each strand.
    pub segments: Vec<u32>,
    /// Points for each segment of every strang.
    pub particles: Vec<Vector3>,
    /// The material index for this hair.
    pub material: Option<usize>,
}

impl Hair {
    /// Constructs a new instance of hair.
    pub fn new() -> Self {
        Self {
            name: None,
            segments: Vec::new(),
            particles: Vec::new(),
            material: None,
        }
    }

    /// Constructs a new instance of hair with the given capacity.
    pub fn with_capacity(strands: usize, particles: usize) -> Self {
        Self {
            name: None,
            segments: Vec::with_capacity(strands),
            particles: Vec::with_capacity(particles),
            material: None,
        }
    }

    /// Sets an optional name for this hair.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.name = name.map(|x| x.into());
        self
    }

    /// Sets an optional material for this hair.
    pub fn material(mut self, index: usize) -> Self {
        self.material = Some(index);
        self
    }

    /// Scales the hair by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for particle in &mut self.particles {
            *particle *= factor;
        }
    }

    /// Transforms the hair by the given matrix.
    pub fn transform(&mut self, matrix: &Matrix4x4) {
        for particle in &mut self.particles {
            particle.transform(matrix);
        }
    }

    /// Converts this 3d hair into a 3d mesh.
    pub fn to_mesh(&self) -> Mesh {
        let segments: usize = self.segments.iter().map(|segment| *segment as usize).sum();

        let mut face_buffer = FaceBuffer::with_capacity(segments * 2);
        let mut vertex_buffer = VertexBuffer::with_capacity(segments * 6)
            .uv_layers(0)
            .maximum_influence(0)
            .build();

        let create_normal = |v1: Vector3, v2: Vector3, v3: Vector3| {
            let tangent = (v2 - v1).normalized();

            (v3 - v1).cross(tangent).normalized()
        };

        let mut create_vertex = |position: Vector3, normal: Vector3| {
            let index = vertex_buffer.len();

            vertex_buffer
                .create()
                .set_position(position)
                .set_normal(normal);

            index as u32
        };

        let particle_extrusion = Vector3::new(0.0, 0.0, 0.010);
        let mut particle_offset: usize = 0;

        for segment in &self.segments {
            for _ in 0..*segment {
                let a = self.particles[particle_offset];
                let b = self.particles[particle_offset + 1];

                particle_offset += 1;

                let a_up = a + particle_extrusion;
                let b_up = b + particle_extrusion;

                let normal1 = create_normal(a, b, a_up);
                let normal2 = create_normal(a, b, b_up);

                let a1 = create_vertex(a, normal1);
                let b1 = create_vertex(b, normal1);
                let a_up1 = create_vertex(a_up, normal1);

                let a2 = create_vertex(a, normal2);
                let b2 = create_vertex(b, normal2);
                let b_up2 = create_vertex(b_up, normal2);

                face_buffer.push(Face::new(a1, b1, a_up1));
                face_buffer.push(Face::new(a2, b2, b_up2));
            }

            particle_offset += 1;
        }

        let mut mesh = Mesh::new(face_buffer, vertex_buffer);

        mesh.name = self.name.clone();
        mesh.material = self.material;
        mesh
    }

    /// Validates the hair has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self) {
        let particle_count: usize = self.segments.iter().map(|x| *x as usize + 1).sum();

        if particle_count != self.particles.len() {
            println!(
                "Validation Error: Found invalid hair particle count: {:?} (segments) != {:?} (particles)",
                particle_count,
                self.particles.len()
            );
        }
    }
}
