use std::io;
use std::io::Write;

use crate::SliceExt;

/// A trait that writes vectors to any `Write` type.
pub trait VecWriteExt: Write {
    /// Writes a vector of `R` to the stream.
    fn write_vec<D, R: AsRef<[D]>>(&mut self, array: R) -> Result<(), io::Error>
    where
        D: Copy + 'static;
}

impl<T> VecWriteExt for T
where
    T: Write,
{
    fn write_vec<D, R: AsRef<[D]>>(&mut self, array: R) -> Result<(), io::Error>
    where
        D: Copy + 'static,
    {
        self.write_all(array.as_ref().as_this_slice())
    }
}
