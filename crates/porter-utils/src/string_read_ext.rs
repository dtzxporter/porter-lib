use std::io;
use std::io::Read;

use crate::StructReadExt;
use crate::VecExt;

/// A trait that reads different string types from any `Read` type.
pub trait StringReadExt: Read {
    /// Reads a string with a null terminator.
    fn read_null_terminated_string(&mut self) -> Result<String, io::Error>;
    /// Reads a string with an exact size as specified.
    fn read_sized_string(
        &mut self,
        size: usize,
        null_terminated: bool,
    ) -> Result<String, io::Error>;
    /// Reads a string with an exact size specified with the given prefixed type.
    fn read_prefix_string<P: Copy + 'static>(
        &mut self,
        null_terminated: bool,
    ) -> Result<String, io::Error>
    where
        usize: TryFrom<P>;
}

impl<T> StringReadExt for T
where
    T: Read,
{
    fn read_null_terminated_string(&mut self) -> Result<String, io::Error> {
        let mut buffer = Vec::with_capacity(256);
        let mut scratch: [u8; 1] = [0; 1];

        loop {
            self.read_exact(&mut scratch)?;

            if scratch[0] == 0 {
                break;
            }

            buffer.try_reserve(1)?;
            buffer.push(scratch[0]);
        }

        String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_sized_string(
        &mut self,
        size: usize,
        null_terminator: bool,
    ) -> Result<String, io::Error> {
        let mut buffer: Vec<u8> = Vec::try_new_with_value(0, size)?;

        self.read_exact(&mut buffer)?;

        if null_terminator && size > 0 {
            buffer.truncate(size - 1);
        }

        String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn read_prefix_string<P: Copy + 'static>(
        &mut self,
        null_terminated: bool,
    ) -> Result<String, io::Error>
    where
        usize: TryFrom<P>,
    {
        let size: P = self.read_struct()?;
        let size =
            usize::try_from(size).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;

        self.read_sized_string(size, null_terminated)
    }
}
