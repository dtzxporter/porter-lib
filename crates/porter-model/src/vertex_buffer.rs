use std::fmt;

use porter_math::Vector2;
use porter_math::Vector3;
use porter_utils::VecExt;

use crate::ModelError;
use crate::Vertex;
use crate::VertexColor;
use crate::VertexMut;
use crate::VertexWeight;

/// Utility to compute the stride of each vertex in bytes.
const fn compute_stride(uv_layers: usize, maximum_influence: usize, colors: usize) -> usize {
    // Vector3: Position
    // Vector3: Normal
    // Vector2[self.uv_layers]: UV layer
    // VertexWeight[self.maximum_influence]: Vertex weight
    // VertexColor[self.colors]: Vertex color
    (size_of::<Vector3>() * 2)
        + (size_of::<Vector2>() * uv_layers)
        + (size_of::<VertexWeight>() * maximum_influence)
        + (size_of::<VertexColor>() * colors)
}

// A buffer of vertices for a mesh.
#[derive(Clone)]
pub struct VertexBuffer {
    buffer: Vec<u8>,
    colors: usize,
    uv_layers: usize,
    maximum_influence: usize,
}

/// Used to build a buffer of vertices based on the configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct VertexBufferBuilder {
    pub(crate) capacity: usize,
    pub(crate) colors: usize,
    pub(crate) uv_layers: usize,
    pub(crate) maximum_influence: usize,
}

impl VertexBuffer {
    /// Creates a new vertex buffer builder.
    pub fn builder() -> VertexBufferBuilder {
        Default::default()
    }

    /// Creates a new vertex buffer builder with the given capacity of vertices.
    pub fn with_capacity(capacity: usize) -> VertexBufferBuilder {
        VertexBufferBuilder {
            capacity,
            ..Default::default()
        }
    }

    /// Creates a new vertex in the buffer.
    #[track_caller]
    pub fn create(&mut self) -> VertexMut<'_> {
        self.buffer
            .resize(self.buffer.len() + self.stride(), 0);

        self.vertex_mut(self.len() - 1)
    }

    /// Tries to create a new vertex in the buffer.
    pub fn try_create(&mut self) -> Result<VertexMut<'_>, ModelError> {
        let stride = self.stride();

        self.buffer
            .try_resize(self.buffer.len() + stride, 0)?;

        Ok(self.vertex_mut(self.len() - 1))
    }

    /// Removes the vertex at the given index.
    #[track_caller]
    pub fn remove(&mut self, index: usize) {
        debug_assert!(index < self.len());

        let stride = self.stride();

        self.buffer
            .drain(index * stride..(index * stride) + stride);
    }

    /// Returns the number of uv layers.
    pub const fn uv_layers(&self) -> usize {
        self.uv_layers
    }

    /// Returns the maximum number of weights.
    pub const fn maximum_influence(&self) -> usize {
        self.maximum_influence
    }

    /// Returns the number of color layers.
    pub const fn colors(&self) -> usize {
        self.colors
    }

    /// Clears the vertex buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.buffer.shrink_to_fit();
    }

    /// Whether or not the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Returns the number of vertices in the buffer.
    pub const fn len(&self) -> usize {
        self.buffer.len() / self.stride()
    }

    /// The stride in bytes of each vertex.
    pub const fn stride(&self) -> usize {
        compute_stride(self.uv_layers, self.maximum_influence, self.colors)
    }

    /// Gets the vertex at the given index.
    #[track_caller]
    pub const fn vertex(&self, index: usize) -> Vertex<'_> {
        debug_assert!(index < self.len());

        Vertex::new(self, index)
    }

    /// Gets a mutable vertex at the given index.
    #[track_caller]
    pub const fn vertex_mut(&mut self, index: usize) -> VertexMut<'_> {
        debug_assert!(index < self.len());

        VertexMut::new(self, index)
    }

    /// Returns the internal buffer used by this vertex buffer.
    pub const fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

impl VertexBufferBuilder {
    /// Sets the maximum number of color layers per vertex.
    #[inline]
    pub const fn colors(mut self, colors: usize) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the maximum number of uv layers per vertex.
    #[inline]
    pub const fn uv_layers(mut self, uv_layers: usize) -> Self {
        self.uv_layers = uv_layers;
        self
    }

    /// Sets the maximum influence per vertex (Maximum number of bones assigned to each vertex for weights).
    #[inline]
    pub const fn maximum_influence(mut self, maximum_influence: usize) -> Self {
        self.maximum_influence = maximum_influence;
        self
    }

    /// Builds the vertex buffer.
    #[inline]
    pub fn build(self) -> VertexBuffer {
        let stride = compute_stride(self.uv_layers, self.maximum_influence, self.colors);

        VertexBuffer {
            buffer: Vec::with_capacity(self.capacity * stride),
            colors: self.colors,
            uv_layers: self.uv_layers,
            maximum_influence: self.maximum_influence,
        }
    }

    /// Attempts to build the vertex buffer.
    #[inline]
    pub fn try_build(self) -> Result<VertexBuffer, ModelError> {
        let stride = compute_stride(self.uv_layers, self.maximum_influence, self.colors);

        Ok(VertexBuffer {
            #[allow(unstable_name_collisions)]
            buffer: Vec::try_with_capacity(self.capacity * stride)?,
            colors: self.colors,
            uv_layers: self.uv_layers,
            maximum_influence: self.maximum_influence,
        })
    }
}

impl fmt::Debug for VertexBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();

        for i in 0..self.len() {
            debug_list.entry(&self.vertex(i));
        }

        debug_list.finish()
    }
}
