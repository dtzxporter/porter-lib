use std::io;
use std::io::Read;

/// A trait that reads structs from `Read` sources.
pub trait StructReadExt: Read {
    /// Reads the type from the reader and advances the stream.
    fn read_struct<S: Copy + 'static>(&mut self) -> Result<S, io::Error>;
    /// Reads a byte length integer from the reader and advances the stream.
    fn read_sized_integer(&mut self, size: usize) -> Result<u64, io::Error>;
}

impl<T> StructReadExt for T
where
    T: Read,
{
    fn read_struct<S: Copy + 'static>(&mut self) -> Result<S, io::Error> {
        let mut result = std::mem::MaybeUninit::<S>::zeroed();

        // SAFETY: This slice has the same length as T, and T is always Copy.
        let slice = unsafe {
            std::slice::from_raw_parts_mut(result.as_mut_ptr() as *mut u8, std::mem::size_of::<S>())
        };

        self.read_exact(slice)?;

        // SAFETY: As long as `read_exact` is safe, we can assume that the full data was initialized.
        Ok(unsafe { result.assume_init() })
    }

    fn read_sized_integer(&mut self, size: usize) -> Result<u64, io::Error> {
        let mut result: u64 = 0;

        debug_assert!(size <= std::mem::size_of::<u64>());

        for i in 0..size {
            result |= (self.read_struct::<u8>()? as u64) << (i * std::mem::size_of::<u64>());
        }

        Ok(result)
    }
}
