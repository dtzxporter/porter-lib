use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use crate::AsAligned;
use crate::StructReadExt;

/// Utility methods for working with seekable streams.
pub trait SeekExt: Seek {
    /// Aligns the current stream position to the given alignment.
    fn align_position(&mut self, alignment: u64) -> io::Result<u64>;
    /// Skips over the given number of bytes from the current position.
    fn skip<P: Copy + 'static>(&mut self, size: P) -> io::Result<u64>
    where
        u64: TryFrom<P>;
    /// Reads a prefixed size, then, skips over that amount of data from the current position.
    fn skip_prefix<P: Copy + 'static>(&mut self) -> io::Result<u64>
    where
        Self: Read,
        u64: TryFrom<P>;
    /// Resets a stream back to the start, then jumps to the given offset.
    fn reset_to<P: Copy + 'static>(&mut self, offset: P) -> io::Result<u64>
    where
        u64: TryFrom<P>;
}

impl<T> SeekExt for T
where
    T: Seek,
{
    fn align_position(&mut self, alignment: u64) -> io::Result<u64> {
        let position = self.stream_position()?;

        self.seek(SeekFrom::Start(position.as_aligned(alignment)))
    }

    fn skip<P: Copy + 'static>(&mut self, size: P) -> io::Result<u64>
    where
        u64: TryFrom<P>,
    {
        let size = u64::try_from(size).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        self.seek(SeekFrom::Current(size as i64))
    }

    fn skip_prefix<P: Copy + 'static>(&mut self) -> io::Result<u64>
    where
        Self: Read,
        u64: TryFrom<P>,
    {
        let size: P = self.read_struct()?;

        self.skip(size)
    }

    fn reset_to<P: Copy + 'static>(&mut self, offset: P) -> io::Result<u64>
    where
        u64: TryFrom<P>,
    {
        let offset =
            u64::try_from(offset).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

        self.seek(SeekFrom::Start(offset))
    }
}
