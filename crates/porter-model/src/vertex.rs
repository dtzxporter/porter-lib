use std::collections::BTreeMap;
use std::fmt;

use porter_math::Vector2;
use porter_math::Vector3;

use crate::VertexBuffer;
use crate::VertexColor;
use crate::VertexWeight;
use crate::WeightBoneId;

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
    pub(crate) fn new(buffer: &'a VertexBuffer, index: usize) -> Self {
        Self { buffer, index }
    }

    /// Returns the position of this vertex.
    pub fn position(&self) -> Vector3 {
        self.read(0)
    }

    /// Returns the normal of this vertex.
    pub fn normal(&self) -> Vector3 {
        self.read(std::mem::size_of::<Vector3>())
    }

    /// Returns the uv layer of this vertex.
    pub fn uv(&self, index: usize) -> Vector2 {
        debug_assert!(index < self.buffer.uv_layers());

        self.read((std::mem::size_of::<Vector3>() * 2) + (std::mem::size_of::<Vector2>() * index))
    }

    /// Returns a weight for this vertex.
    pub fn weight(&self, index: usize) -> VertexWeight {
        debug_assert!(index < self.buffer.maximum_influence());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * index),
        )
    }

    /// Returns the unique weights for this vertex.
    pub fn unique_weights(&self) -> BTreeMap<WeightBoneId, f32> {
        let mut result = BTreeMap::new();

        for w in 0..self.buffer.maximum_influence() {
            let weight = self.weight(w);

            *result.entry(weight.bone).or_default() += weight.value;
        }

        result
    }

    /// Returns the color for this vertex.
    pub fn color(&self, index: usize) -> VertexColor {
        debug_assert!(index < self.buffer.colors());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * self.buffer.maximum_influence())
                + (std::mem::size_of::<VertexColor>() * index),
        )
    }

    /// Reads T from the specified offset.
    #[inline(always)]
    fn read<T: Copy>(&self, offset: usize) -> T {
        let count = (self.index * self.buffer.stride()) + offset;

        debug_assert!((count + std::mem::size_of::<T>()) <= self.buffer.as_slice().len());

        // SAFETY: We assert that the count of bytes is less than the buffer size.
        unsafe { std::ptr::read(self.buffer.as_slice().as_ptr().add(count) as *const T) }
    }
}

impl<'a> VertexMut<'a> {
    /// Creates a new instance of vertex.
    pub(crate) fn new(buffer: &'a mut VertexBuffer, index: usize) -> Self {
        Self { buffer, index }
    }

    /// Returns the position of this vertex.
    pub fn position(&self) -> Vector3 {
        self.read(0)
    }

    /// Sets the position of this vertex.
    pub fn set_position(&mut self, position: Vector3) -> &mut Self {
        self.write(0, position);
        self
    }

    /// Returns the normal of this vertex.
    pub fn normal(&self) -> Vector3 {
        self.read(std::mem::size_of::<Vector3>())
    }

    /// Sets the normal of this vertex.
    pub fn set_normal(&mut self, normal: Vector3) -> &mut Self {
        self.write(std::mem::size_of::<Vector3>(), normal);
        self
    }

    /// Returns the uv layer of this vertex.
    pub fn uv(&self, index: usize) -> Vector2 {
        debug_assert!(index < self.buffer.uv_layers());

        self.read((std::mem::size_of::<Vector3>() * 2) + (std::mem::size_of::<Vector2>() * index))
    }

    /// Sets the uv layer of this vertex.
    pub fn set_uv(&mut self, index: usize, value: Vector2) -> &mut Self {
        debug_assert!(index < self.buffer.uv_layers());

        self.write(
            (std::mem::size_of::<Vector3>() * 2) + (std::mem::size_of::<Vector2>() * index),
            value,
        );

        self
    }

    /// Returns a weight for this vertex.
    pub fn weight(&self, index: usize) -> VertexWeight {
        debug_assert!(index < self.buffer.maximum_influence());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * index),
        )
    }

    /// Returns the unique weights for this vertex.
    pub fn unique_weights(&self) -> BTreeMap<WeightBoneId, f32> {
        let mut result = BTreeMap::new();

        for w in 0..self.buffer.maximum_influence() {
            let weight = self.weight(w);

            *result.entry(weight.bone).or_default() += weight.value;
        }

        result
    }

    /// Sets a weight for this vertex.
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

    /// Sets the weight bone index for this vertex.
    pub fn set_weight_bone(&mut self, index: usize, bone: WeightBoneId) -> &mut Self {
        let mut weight = self.weight(index);
        weight.bone = bone;

        self.set_weight(index, weight);

        self
    }

    /// Sets the weight bone value for this vertex.
    pub fn set_weight_value(&mut self, index: usize, value: f32) -> &mut Self {
        let mut weight = self.weight(index);
        weight.value = value;

        self.set_weight(index, weight);

        self
    }

    /// Returns the color for this vertex.
    pub fn color(&self, index: usize) -> VertexColor {
        debug_assert!(index < self.buffer.colors());

        self.read(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * self.buffer.maximum_influence())
                + (std::mem::size_of::<VertexColor>() * index),
        )
    }

    /// Returns the color for this vertex.
    pub fn set_color(&mut self, index: usize, color: VertexColor) -> &mut Self {
        debug_assert!(index < self.buffer.colors());

        self.write(
            (std::mem::size_of::<Vector3>() * 2)
                + (std::mem::size_of::<Vector2>() * self.buffer.uv_layers())
                + (std::mem::size_of::<VertexWeight>() * self.buffer.maximum_influence())
                + (std::mem::size_of::<VertexColor>() * index),
            color,
        );

        self
    }

    /// Copies all of the values from the given vertex.
    pub fn copy_from(&mut self, vertex: &Vertex<'_>) {
        debug_assert!(self.buffer.colors() == vertex.buffer.colors());
        debug_assert!(self.buffer.uv_layers() == vertex.buffer.uv_layers());
        debug_assert!(self.buffer.maximum_influence() == vertex.buffer.maximum_influence());

        let size = self.buffer.stride();

        let offset_src = vertex.index * size;
        let offset_dst = self.index * size;

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertex.buffer.as_slice().as_ptr().add(offset_src),
                self.buffer.as_slice().as_ptr().add(offset_dst) as *mut u8,
                size,
            )
        };
    }

    /// Copies all of the values from the given vertex.
    pub fn copy_from_mut(&mut self, vertex: &VertexMut<'_>) {
        debug_assert!(self.buffer.colors() == vertex.buffer.colors());
        debug_assert!(self.buffer.uv_layers() == vertex.buffer.uv_layers());
        debug_assert!(self.buffer.maximum_influence() == vertex.buffer.maximum_influence());
        debug_assert!(self.buffer.stride() == vertex.buffer.stride());

        let size = self.buffer.stride();

        let offset_src = vertex.index * size;
        let offset_dst = self.index * size;

        unsafe {
            std::ptr::copy_nonoverlapping(
                vertex.buffer.as_slice().as_ptr().add(offset_src),
                self.buffer.as_slice().as_ptr().add(offset_dst) as *mut u8,
                size,
            )
        };
    }

    /// Reads T from the specified offset.
    #[inline(always)]
    fn read<T: Copy>(&self, offset: usize) -> T {
        let count = (self.index * self.buffer.stride()) + offset;

        debug_assert!((count + std::mem::size_of::<T>()) <= self.buffer.as_slice().len());

        // SAFETY: We assert that the count of bytes is less than the buffer size.
        unsafe { std::ptr::read(self.buffer.as_slice().as_ptr().add(count) as *const T) }
    }

    /// Writes T to the specified offset.
    #[inline(always)]
    fn write<T: Copy>(&mut self, offset: usize, value: T) {
        let count = (self.index * self.buffer.stride()) + offset;

        debug_assert!((count + std::mem::size_of::<T>()) <= self.buffer.as_slice().len());

        // SAFETY: We assert that the count of bytes is less than the buffer size.
        unsafe { std::ptr::write(self.buffer.as_slice().as_ptr().add(count) as *mut T, value) };
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

        for i in 0..self.buffer.colors() {
            debug.field(&format!("color[{}]", i), &self.color(i));
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

        for i in 0..self.buffer.colors() {
            debug.field(&format!("color[{}]", i), &self.color(i));
        }

        debug.finish()
    }
}
