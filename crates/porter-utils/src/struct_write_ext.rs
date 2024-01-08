use std::io;
use std::io::Write;

use crate::AsByteSlice;

/// A trait that writes structs to `Write` destinations.
pub trait StructWriteExt: Write {
    /// Writes the type to the writer and advances the stream.
    fn write_struct<S: Copy + 'static>(&mut self, value: S) -> Result<(), io::Error>;
    /// Writes a byte length integer to the writer and advances the stream.
    fn write_sized_integer(&mut self, value: u64, size: usize) -> Result<(), io::Error>;
}

impl<T> StructWriteExt for T
where
    T: Write,
{
    fn write_struct<S: Copy + 'static>(&mut self, value: S) -> Result<(), io::Error> {
        self.write_all(value.as_byte_slice())
    }

    fn write_sized_integer(&mut self, value: u64, size: usize) -> Result<(), io::Error> {
        debug_assert!(size <= std::mem::size_of::<u64>());

        for i in 0..size {
            self.write_struct::<u8>(((value >> (i * std::mem::size_of::<u64>())) & 0xFF) as u8)?;
        }

        Ok(())
    }
}
