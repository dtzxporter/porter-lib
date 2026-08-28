use porter_utils::VecExt;

use crate::TextureError;

/// A single frame of an image.
#[derive(Debug, Clone)]
pub struct Frame {
    buffer: Vec<u8>,
}

impl Frame {
    /// Allocates a new frame with the given buffer size.
    #[inline]
    pub(crate) fn new(size: u32) -> Result<Self, TextureError> {
        Ok(Frame {
            buffer: Vec::try_new_zeroed(size as _)?,
        })
    }

    /// Creates a new frame with the given buffer.
    #[inline]
    pub(crate) const fn with_buffer(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }

    /// Swaps out the internal buffer for the given one.
    #[inline]
    pub(crate) fn replace_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
    }

    /// Truncates the internal buffer to the new length.
    #[inline]
    pub(crate) fn truncate_buffer(&mut self, length: usize) {
        self.buffer.truncate(length);
    }

    /// Returns an immutable slice of the frame buffer.
    #[inline]
    pub const fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// Returns the frame buffer as a mutable slice.
    #[inline]
    pub const fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }

    /// Fill the frame buffer with the given value.
    pub fn fill<const SIZE: usize>(&mut self, value: [u8; SIZE]) {
        for chunk in self.buffer.chunks_exact_mut(SIZE) {
            let chunk: &mut [u8; SIZE] = chunk
                .try_into()
                // This can never fail.
                .unwrap();

            *chunk = value;
        }
    }
}
