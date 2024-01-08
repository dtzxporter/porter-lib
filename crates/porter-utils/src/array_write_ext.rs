use std::io;
use std::io::Write;

use crate::AsThisSlice;

/// A trait that writes arrays to any `Write` type.
pub trait ArrayWriteExt: Write {
    /// Writes an array of `R` with the given length.
    fn write_array<D, R: AsRef<[D]>>(&mut self, array: R) -> Result<(), io::Error>
    where
        D: Copy + 'static;
}

impl<T> ArrayWriteExt for T
where
    T: Write,
{
    fn write_array<D, R: AsRef<[D]>>(&mut self, array: R) -> Result<(), io::Error>
    where
        D: Copy + 'static,
    {
        self.write_all(array.as_ref().as_this_slice())
    }
}
