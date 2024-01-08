use crate::FaceBuffer;
use crate::VertexBuffer;

/// A polygon mesh for a model.
#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: Option<String>,
    pub faces: FaceBuffer,
    pub vertices: VertexBuffer,
    pub materials: Vec<isize>,
}

impl Mesh {
    /// Constructs a new mesh instance.
    #[inline]
    pub fn new(faces: FaceBuffer, vertices: VertexBuffer) -> Self {
        Self {
            name: None,
            materials: Vec::with_capacity(vertices.uv_layers()),
            faces,
            vertices,
        }
    }

    /// Sets an optional name for this mesh.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.name = name.map(|x| x.into());
        self
    }

    /// Scales the mesh by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for i in 0..self.vertices.len() {
            let mut vertex = self.vertices.vertex_mut(i);

            vertex.set_position(vertex.position() * factor);
        }
    }

    /// Validates the mesh has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self, bone_count: usize) {
        for v in 0..self.vertices.len() {
            let vertex = self.vertices.vertex(v);

            let normal = vertex.normal();
            let length_squared = normal.length_squared();

            if !(0.98..=1.025).contains(&length_squared) {
                println!(
                    "Validation Error: Found normal with non-1.0 square sum: {} - {}, {}, {} [{}]",
                    length_squared, normal.x, normal.y, normal.z, v
                );
            }

            let mut total = 0.0;

            for w in 0..self.vertices.maximum_influence() {
                let weight = vertex.weight(w);

                if weight.bone >= bone_count as u16 {
                    println!(
                        "Validation Error: Found weight bone outside of skeleton: {} [{}:{}]",
                        { weight.bone },
                        v,
                        w
                    );
                }

                total += weight.value;
            }

            if self.vertices.maximum_influence() > 0 && !(0.9825..=1.125).contains(&total) {
                println!(
                    "Validation Error: Found weight doesn't add up to 1.0: {} [{}]",
                    total, v
                );
            }
        }

        let vertex_count = self.vertices.len() as u32;

        for (index, face) in self.faces.iter().enumerate() {
            if face.i1 < vertex_count && face.i2 < vertex_count && face.i3 < vertex_count {
                continue;
            }

            println!(
                "Validate Error: Found face with invalid indexes: {}, {}, {} [{}]",
                { face.i1 },
                { face.i2 },
                { face.i3 },
                index
            );
        }
    }
}
