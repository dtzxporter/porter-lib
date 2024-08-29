use std::io;
use std::io::Write;

use crate::StructWriteExt;

/// A trait that writes different string types to any `Write` type.
pub trait StringWriteExt: Write {
    /// Writes a string with a null terminator.
    fn write_null_terminated_string<S: AsRef<str>>(&mut self, string: S) -> Result<(), io::Error>;
    /// Writes a string without a null terminator.
    fn write_string<S: AsRef<str>>(&mut self, string: S) -> Result<(), io::Error>;
    /// Writes a string with a prefixed size before it.
    fn write_prefix_string<P, S: AsRef<str>>(
        &mut self,
        string: S,
        null_terminated: bool,
    ) -> Result<(), io::Error>
    where
        P: TryFrom<usize> + Copy + 'static;
}

impl<T> StringWriteExt for T
where
    T: Write,
{
    fn write_null_terminated_string<S: AsRef<str>>(&mut self, string: S) -> Result<(), io::Error> {
        self.write_all(string.as_ref().as_bytes())?;
        self.write_all(&[0])
    }

    fn write_string<S: AsRef<str>>(&mut self, string: S) -> Result<(), io::Error> {
        self.write_all(string.as_ref().as_bytes())
    }

    fn write_prefix_string<P, S: AsRef<str>>(
        &mut self,
        string: S,
        null_terminated: bool,
    ) -> Result<(), io::Error>
    where
        P: TryFrom<usize> + Copy + 'static,
    {
        let string = string.as_ref();
        let size = if null_terminated {
            string.len() + 1
        } else {
            string.len()
        };

        let size: P =
            P::try_from(size).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;

        self.write_struct(size)?;

        if null_terminated {
            self.write_null_terminated_string(string)
        } else {
            self.write_string(string)
        }
    }
}
