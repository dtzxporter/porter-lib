use std::io::BufReader;
use std::io::Read;

/// Utility trait used to add buffered reading extensions.
pub trait BufferReadExt: Read {
    /// Wraps this [`Read`] object in a [`BufReader`].
    fn buffer_read(self) -> BufReader<Self>;
    /// Wraps this [`Read`] object in a [`BufReader`] with the given capacity.
    fn buffer_read_with(self, capacity: usize) -> BufReader<Self>;
}

impl<T> BufferReadExt for T
where
    T: Read,
{
    #[inline]
    fn buffer_read(self) -> BufReader<Self> {
        BufReader::new(self)
    }

    #[inline]
    fn buffer_read_with(self, capacity: usize) -> BufReader<Self> {
        BufReader::with_capacity(capacity, self)
    }
}
