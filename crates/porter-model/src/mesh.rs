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

    /// Sets the name of this mesh.
    pub fn name<S: Into<String>>(&mut self, name: S) {
        self.name = Some(name.into());
    }

    /// Scales the mesh by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for i in 0..self.vertices.len() {
            let mut vertex = self.vertices.vertex_mut(i);

            vertex.set_position(vertex.position() * factor);
        }
    }
}
