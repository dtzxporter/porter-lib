use std::collections::BTreeMap;

use porter_math::Matrix4x4;
use porter_math::Vector3;

/// A 3d blend shape.
#[derive(Debug, Clone)]
pub struct BlendShape {
    pub name: String,
    pub vertex_deltas: BTreeMap<u32, Vector3>,
    pub target_scale: f32,
}

impl BlendShape {
    /// Constructs a new instance of blend shape.
    pub fn new(name: String) -> Self {
        Self {
            name,
            vertex_deltas: BTreeMap::new(),
            target_scale: 1.0,
        }
    }

    /// Scales the blend shape by the given factor.
    pub fn scale(&mut self, factor: f32) {
        for position in self.vertex_deltas.values_mut() {
            *position *= factor;
        }
    }

    /// Transforms the blend shape by the given matrix.
    pub fn transform(&mut self, matrix: &Matrix4x4) {
        let scale = matrix.scale();

        for vertex_delta in self.vertex_deltas.values_mut() {
            *vertex_delta *= scale;
        }
    }

    /// Sets the target scale value.
    pub fn target_scale(mut self, target_scale: f32) -> Self {
        self.target_scale = target_scale;
        self
    }

    /// Validates the blend shape has some form of valid data.
    #[cfg(debug_assertions)]
    pub fn validate(&self, vertex_count: usize) {
        if self.vertex_deltas.len() > vertex_count {
            println!(
                "Validate Error: Found invalid blend shape too much data: {:?}",
                self.name
            );
            return;
        }

        for (index, vertex_index) in self.vertex_deltas.keys().enumerate() {
            if *vertex_index as usize >= vertex_count {
                println!(
                    "Validate Error: Found invalid blend shape index: {:?} [{}:{}]",
                    vertex_index, self.name, index
                );
            }
        }
    }
}
