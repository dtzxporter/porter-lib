/// A material remap operation by faces.
#[derive(Debug)]
pub struct MaterialRemapFaces {
    pub(crate) material: String,
    pub(crate) mesh: usize,
    pub(crate) face_start: usize,
    pub(crate) length: usize,
}

/// A material remap operation by vertex.
#[derive(Debug)]
pub struct MaterialRemapVertices {
    pub(crate) material: String,
    pub(crate) mesh: usize,
    pub(crate) vertex_start: usize,
    pub(crate) length: usize,
}

impl MaterialRemapFaces {
    /// Constructs a new instance of material remap by faces.
    pub fn new(material: String, mesh: usize, face_start: usize, length: usize) -> Self {
        Self {
            material,
            mesh,
            face_start,
            length,
        }
    }
}

impl MaterialRemapVertices {
    /// Constructs a new instance of material remap by vertex.
    pub fn new(material: String, mesh: usize, vertex_start: usize, length: usize) -> Self {
        Self {
            material,
            mesh,
            vertex_start,
            length,
        }
    }
}
