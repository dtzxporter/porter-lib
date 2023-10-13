/// A 3d blend shape.
#[derive(Debug, Clone)]
pub struct BlendShape {
    pub name: Option<String>,
    pub base_mesh: usize,
    pub target_meshes: Vec<usize>,
    pub target_scales: Vec<f32>,
}

impl BlendShape {
    /// Constructs a new instance of blend shape.
    #[inline]
    pub fn new(name: Option<String>, base_mesh: usize) -> Self {
        Self {
            name,
            base_mesh,
            target_meshes: Vec::new(),
            target_scales: Vec::new(),
        }
    }

    /// Adds a target to the blend shape.
    pub fn add_target(mut self, target_mesh: usize, target_scale: f32) -> Self {
        self.target_meshes.push(target_mesh);
        self.target_scales.push(target_scale);
        self
    }
}
