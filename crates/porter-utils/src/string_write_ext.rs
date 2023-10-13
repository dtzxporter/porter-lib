use std::io;
use std::io::Write;

/// A trait that writes different string types to any `Write` type.
pub trait StringWriteExt: Write {
    /// Writes a string with a null terminator.
    fn write_null_terminated_string<S: AsRef<str>>(&mut self, string: S) -> Result<(), io::Error>;
    /// Writes a string without a null terminator.
    fn write_string<S: AsRef<str>>(&mut self, string: S) -> Result<(), io::Error>;
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
}
