use std::fmt;

use porter_math::Vector2;
use porter_math::Vector3;

use crate::VertexBuffer;
use crate::VertexColor;
use crate::VertexWeight;

/// A single vertex of a polygon mesh.
pub struct Vertex<'a> {
    buffer: &'a VertexBuffer,
    index: usize,
}

/// A mutable single vertex of a polygon mesh.
pub struct VertexMut<'a> {
    buffer: &'a mut VertexBuffer,
    index: usize,
}

impl<'a> Vertex<'a> {
    /// Creates a new instance of vertex.
    #[inline]
    pub(crate) fn new(buffer: &'a VertexBuffer, index: usize) -> Self {
        Self { buffer, index }
    }

    /// Returns the position of this vertex.
    #[inline]
    pub fn position(&self) -> Vector3 {
        self.read(0)
    }

    /// Returns the normal of this vertex.
    #[inline]
    pub fn normal(&self) -> Vector3 {
        self.read(std::mem::size_of::<Vector3>())
    }

    /// Returns the uv layer of this vertex.
    #[inline]
    pub fn uv(&self, index: usize) -> Vector2 {
        debug_assert!(index < self.buffer.uv_layers());

        self.read((std::mem::size_of::<Vector3>() * 2) + (std::mem::size_of::<Vector2>() * index))
    }

    /// Returns a weight for this vertex.
    #[inline]
    pub fn weight(&self, index: usize) -> VertexWeight {
        debug_assert!(index < self.buffer.maximum_influence());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * index),
        )
    }

    /// Returns the count of non-zero weight values.
    #[inline]
    pub fn weight_count(&self) -> usize {
        let mut count = 0;

        for i in 0..self.buffer.maximum_influence() {
            if self.weight(i).value != 0.0 {
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    /// Returns the color for this vertex.
    #[inline]
    pub fn color(&self) -> VertexColor {
        debug_assert!(self.buffer.colors());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * self.buffer.maximum_influence()),
        )
    }

    /// Reads T from the specified offset.
    #[inline(always)]
    fn read<T: Copy>(&self, offset: usize) -> T {
        let count = (self.index * self.buffer.stride()) + offset;

        debug_assert!(count < self.buffer.as_slice().len());

        unsafe { std::ptr::read(self.buffer.as_slice().as_ptr().add(count) as *const T) }
    }
}

impl<'a> VertexMut<'a> {
    /// Creates a new instance of vertex.
    #[inline]
    pub(crate) fn new(buffer: &'a mut VertexBuffer, index: usize) -> Self {
        Self { buffer, index }
    }

    /// Returns the position of this vertex.
    #[inline]
    pub fn position(&self) -> Vector3 {
        self.read(0)
    }

    /// Sets the position of this vertex.
    #[inline]
    pub fn set_position(&mut self, position: Vector3) -> &mut Self {
        self.write(0, position);
        self
    }

    /// Returns the normal of this vertex.
    #[inline]
    pub fn normal(&self) -> Vector3 {
        self.read(std::mem::size_of::<Vector3>())
    }

    /// Sets the normal of this vertex.
    #[inline]
    pub fn set_normal(&mut self, normal: Vector3) -> &mut Self {
        self.write(std::mem::size_of::<Vector3>(), normal);
        self
    }

    /// Returns the uv layer of this vertex.
    #[inline]
    pub fn uv(&self, index: usize) -> Vector2 {
        debug_assert!(index < self.buffer.uv_layers());

        self.read((std::mem::size_of::<Vector3>() * 2) + (std::mem::size_of::<Vector2>() * index))
    }

    /// Sets the uv layer of this vertex.
    #[inline]
    pub fn set_uv(&mut self, index: usize, value: Vector2) -> &mut Self {
        debug_assert!(index < self.buffer.uv_layers());

        self.write(
            (std::mem::size_of::<Vector3>() * 2) + (std::mem::size_of::<Vector2>() * index),
            value,
        );

        self
    }

    /// Returns a weight for this vertex.
    #[inline]
    pub fn weight(&self, index: usize) -> VertexWeight {
        debug_assert!(index < self.buffer.maximum_influence());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * index),
        )
    }

    /// Returns the count of non-zero weight values.
    #[inline]
    pub fn weight_count(&self) -> usize {
        let mut count = 0;

        for i in 0..self.buffer.maximum_influence() {
            if self.weight(i).value != 0.0 {
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    /// Sets a weight for this vertex.
    #[inline]
    pub fn set_weight(&mut self, index: usize, weight: VertexWeight) -> &mut Self {
        debug_assert!(index < self.buffer.maximum_influence());

        self.write(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * index),
            weight,
        );

        self
    }

    /// Returns the color for this vertex.
    #[inline]
    pub fn color(&self) -> VertexColor {
        debug_assert!(self.buffer.colors());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * self.buffer.maximum_influence()),
        )
    }

    /// Returns the color for this vertex.
    #[inline]
    pub fn set_color(&mut self, color: VertexColor) -> &mut Self {
        debug_assert!(self.buffer.colors());

        self.write(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * self.buffer.maximum_influence()),
            color,
        );

        self
    }

    /// Reads T from the specified offset.
    #[inline(always)]
    fn read<T: Copy>(&self, offset: usize) -> T {
        let count = (self.index * self.buffer.stride()) + offset;

        debug_assert!(count < self.buffer.as_slice().len());

        unsafe { std::ptr::read(self.buffer.as_slice().as_ptr().add(count) as *const T) }
    }

    /// Writes T to the specified offset.
    #[inline(always)]
    fn write<T: Copy>(&mut self, offset: usize, value: T) {
        let count = (self.index * self.buffer.stride()) + offset;

        debug_assert!(count < self.buffer.as_slice().len());

        unsafe {
            std::ptr::write(self.buffer.as_slice().as_ptr().add(count) as *mut T, value);
        }
    }
}

impl<'a> fmt::Debug for Vertex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Vertex");

        debug
            .field("position", &self.position())
            .field("normal", &self.normal());

        for i in 0..self.buffer.uv_layers() {
            debug.field(&format!("uv_layer[{}]", i), &self.uv(i));
        }

        for i in 0..self.buffer.maximum_influence() {
            debug.field(&format!("weight[{}]", i), &self.weight(i));
        }

        if self.buffer.colors() {
            debug.field("color", &self.color());
        }

        debug.finish()
    }
}

impl<'a> fmt::Debug for VertexMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Vertex");

        debug
            .field("position", &self.position())
            .field("normal", &self.normal());

        for i in 0..self.buffer.uv_layers() {
            debug.field(&format!("uv_layer[{}]", i), &self.uv(i));
        }

        for i in 0..self.buffer.maximum_influence() {
            debug.field(&format!("weight[{}]", i), &self.weight(i));
        }

        if self.buffer.colors() {
            debug.field("color", &self.color());
        }

        debug.finish()
    }
}
