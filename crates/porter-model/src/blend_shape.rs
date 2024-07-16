use porter_math::Vector3;

/// A 3d blend shape.
#[derive(Debug, Clone)]
pub struct BlendShape {
    pub name: String,
    pub base_mesh: usize,
    pub vertex_indices: Vec<u32>,
    pub vertex_positions: Vec<Vector3>,
    pub target_scale: f32,
}

impl BlendShape {
    /// Constructs a new instance of blend shape.
    pub fn new(name: String, base_mesh: usize) -> Self {
        Self {
            name,
            base_mesh,
            vertex_indices: Vec::new(),
            vertex_positions: Vec::new(),
            target_scale: 1.0,
        }
    }

    /// Scales the blend shape by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for position in &mut self.vertex_positions {
            *position *= factor;
        }
    }

    /// Sets the target scale value.
    pub fn target_scale(mut self, target_scale: f32) -> Self {
        self.target_scale = target_scale;
        self
    }
}
