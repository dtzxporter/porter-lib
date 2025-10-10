use crate::TextureError;

/// A single frame of an image.
#[derive(Debug, Clone)]
pub struct Frame {
    buffer: Vec<u8>,
}

impl Frame {
    /// Allocates a new frame with the given dimensions and buffer size.
    pub(crate) fn new(size: u32) -> Result<Frame, TextureError> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer
            .try_reserve_exact(size as usize)
            .map_err(|_| TextureError::FrameAllocationFailed)?;

        buffer.resize(size as usize, 0);

        Ok(Frame { buffer })
    }

    /// Swaps out the internal buffer for the given one.
    pub(crate) fn replace_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = buffer;
    }

    /// Truncates the internal buffer to the new length.
    pub(crate) fn truncate_buffer(&mut self, length: usize) {
        self.buffer.truncate(length);
    }

    /// Returns an immutable slice of the frame buffer.
    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns the frame buffer as a mutable slice.
    #[inline(always)]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}
