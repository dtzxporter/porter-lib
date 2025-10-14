use std::io;
use std::io::Read;
use std::mem::ManuallyDrop;
use std::mem::MaybeUninit;
use std::slice::from_raw_parts_mut;

use crate::VecExt;

/// A trait that reads arrays from any `Read` type.
pub trait ArrayReadExt: Read {
    /// Reads an array of `R` with the given length.
    fn read_array<R>(&mut self, length: usize) -> Result<Vec<R>, io::Error>
    where
        R: Copy + 'static;

    /// Reads an array of `u8` until EOF.
    fn read_array_to_end(&mut self) -> Result<Vec<u8>, io::Error>;
}

impl<T> ArrayReadExt for T
where
    T: Read,
{
    fn read_array<R>(&mut self, length: usize) -> Result<Vec<R>, io::Error>
    where
        R: Copy + 'static,
    {
        let mut result: Vec<MaybeUninit<R>> =
            Vec::try_new_with_value(MaybeUninit::<R>::zeroed(), length)?;

        let slice = result.as_mut_slice();
        let bytes = slice.len() * size_of::<R>();

        // SAFETY: We ensure that the size of each element correctly matches the size in bytes.
        let slice = unsafe { from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, bytes) };

        self.read_exact(slice)?;

        let mut result = ManuallyDrop::new(result);

        let ptr = result.as_mut_ptr();
        let len = result.len();
        let cap = result.capacity();

        // SAFETY: The source data was a Vec<> and MaybeUninit always has the same memory layout as R.
        Ok(unsafe { Vec::from_raw_parts(ptr as *mut R, len, cap) })
    }

    fn read_array_to_end(&mut self) -> Result<Vec<u8>, io::Error> {
        let mut result: Vec<u8> = Vec::new();

        self.read_to_end(&mut result)?;

        Ok(result)
    }
}
