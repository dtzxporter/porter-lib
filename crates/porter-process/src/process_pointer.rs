use std::io::Read;
use std::io::Seek;
use std::marker::PhantomData;

use porter_utils::SeekExt;
use porter_utils::StringReadExt;
use porter_utils::StructReadExt;

use crate::ProcessError;

/// An opaque pointer type which allows reading the data which the pointer points to in a process.
#[derive(Debug, Clone, Copy)]
pub struct ProcessPointer<S, T>
where
    S: Copy + 'static,
    T: Copy + 'static,
{
    inner: S,
    _phantom: PhantomData<T>,
}

impl ProcessPointer<u64, &str> {
    /// Reads a null terminated string.
    pub fn read_string<R: Read + Seek>(&self, reader: &mut R) -> Result<String, ProcessError> {
        let current_position = reader.stream_position()?;

        reader.reset_to(self.inner)?;

        let string = reader.read_null_terminated_string()?;

        let _ = reader.reset_to(current_position);

        Ok(string)
    }
}

impl ProcessPointer<u32, &str> {
    /// Reads a null terminated string.
    pub fn read_string<R: Read + Seek>(&self, reader: &mut R) -> Result<String, ProcessError> {
        let current_position = reader.stream_position()?;

        reader.reset_to(self.inner)?;

        let string = reader.read_null_terminated_string()?;

        let _ = reader.reset_to(current_position);

        Ok(string)
    }
}

impl<T> ProcessPointer<u64, T>
where
    T: Copy + 'static,
{
    /// Reads [`T`] from the given reader.
    pub fn read<R: Read + Seek>(&self, reader: &mut R) -> Result<T, ProcessError> {
        let current_position = reader.stream_position()?;

        reader.reset_to(self.inner)?;

        let value: T = reader.read_struct()?;

        let _ = reader.reset_to(current_position);

        Ok(value)
    }
}

impl<T> ProcessPointer<u32, T>
where
    T: Copy + 'static,
{
    /// Reads [`T`] from the given reader.
    pub fn read<R: Read + Seek>(&self, reader: &mut R) -> Result<T, ProcessError> {
        let current_position = reader.stream_position()?;

        reader.reset_to(self.inner)?;

        let value: T = reader.read_struct()?;

        let _ = reader.reset_to(current_position);

        Ok(value)
    }
}
