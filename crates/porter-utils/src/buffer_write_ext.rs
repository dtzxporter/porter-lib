use std::io::BufWriter;
use std::io::Write;

/// Utility trait used to add buffered writing extensions.
pub trait BufferWriteExt: Write {
    /// Wraps this [`Write`] object in a [`BufWriter`].
    fn buffer_write(self) -> BufWriter<Self>;
    /// Wraps this [`Write`] object in a [`BufWriter`] with the given capacity.
    fn buffer_write_with(self, capacity: usize) -> BufWriter<Self>;
}

impl<T> BufferWriteExt for T
where
    T: Write,
{
    #[inline]
    fn buffer_write(self) -> BufWriter<Self> {
        BufWriter::new(self)
    }

    #[inline]
    fn buffer_write_with(self, capacity: usize) -> BufWriter<Self> {
        BufWriter::with_capacity(capacity, self)
    }
}
