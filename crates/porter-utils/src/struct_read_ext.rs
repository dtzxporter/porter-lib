use std::io;
use std::io::Read;

/// A trait that reads structs from different sources.
pub trait StructReadExt: Copy {
    /// Creates a struct from a slice of bytes equal to or greater than it's size.
    fn from_byte_slice<S: AsRef<[u8]>>(slice: S) -> Result<Self, io::Error>;
    /// Reads the type from the reader and advances the stream.
    fn from_io_read<R: Read>(reader: R) -> Result<Self, io::Error>;
}

impl<T> StructReadExt for T
where
    T: Copy,
{
    fn from_byte_slice<S: AsRef<[u8]>>(slice: S) -> Result<Self, io::Error> {
        let slice = slice.as_ref();
        let size = std::mem::size_of::<Self>();

        if slice.len() < size {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }

        Ok(unsafe { std::ptr::read(slice.as_ptr() as *const T) })
    }

    fn from_io_read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let mut buffer: [u8; 512] = [0; 512];

        let size = std::mem::size_of::<Self>();

        if size > buffer.len() {
            return Err(io::Error::from(io::ErrorKind::OutOfMemory));
        }

        reader.read_exact(&mut buffer[0..size])?;

        Self::from_byte_slice(&buffer[0..size])
    }
}
