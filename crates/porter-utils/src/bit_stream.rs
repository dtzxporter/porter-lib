use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use crate::AsAligned;
use crate::BitSink;
use crate::StackVec;

/// A bit stream that provides methods for reading data as bits (Lsb order).
pub struct BitStream<T> {
    source: T,
    buffer: StackVec<u8, 8>,
    buffer_offset: usize,
    msb: bool,
}

impl BitStream<()> {
    /// Constructs a new bit stream from the given vec of bytes in lsb order.
    #[inline]
    pub const fn from_vec_lsb(vec: Vec<u8>) -> BitStream<Cursor<Vec<u8>>> {
        BitStream::new_lsb(Cursor::new(vec))
    }

    /// Constructs a new bit stream from the given vec of bytes in msb order.
    #[inline]
    pub const fn from_vec_msb(vec: Vec<u8>) -> BitStream<Cursor<Vec<u8>>> {
        BitStream::new_msb(Cursor::new(vec))
    }
}

impl<T> BitStream<T> {
    /// Consumes the bit stream and returns the inner source.
    ///
    /// Any leftover data in the internal buffer is lost.
    #[inline]
    pub fn into_inner(self) -> T {
        self.source
    }

    /// Gets a mutable reference to the inner source.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.source
    }

    /// Gets a reference to the inner source.
    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.source
    }
}

impl<T> BitStream<T>
where
    T: Read,
{
    /// Constructs a new bit stream from the given source in lsb order.
    #[inline]
    pub const fn new_lsb(source: T) -> Self {
        Self {
            source,
            buffer: StackVec::new([0; 8]),
            buffer_offset: 0,
            msb: false,
        }
    }

    /// Constructs a new bit stream from the given source in msb order.
    #[inline]
    pub const fn new_msb(source: T) -> Self {
        Self {
            source,
            buffer: StackVec::new([0; 8]),
            buffer_offset: 0,
            msb: true,
        }
    }

    /// Aligns the bit stream to the next byte.
    #[inline]
    pub fn align(&mut self) -> Result<()> {
        self.buffer_offset = self.buffer_offset.as_aligned(8);
        Ok(())
    }

    /// Reads a 1bit boolean value.
    #[inline]
    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u64(1)? > 0)
    }

    /// Reads a signed 8bit integer with the given number of bits.
    #[inline]
    pub fn read_i8(&mut self, bits: u64) -> Result<i8> {
        Ok(i8::from_ne_bytes(self.read_u8(bits)?.to_ne_bytes()))
    }

    /// Reads an unsigned 8bit integer with the given number of bits.
    #[inline]
    pub fn read_u8(&mut self, bits: u64) -> Result<u8> {
        if bits > u8::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        Ok(self.read_u64(bits)? as u8)
    }

    /// Reads a signed 16bit integer with the given number of bits.
    #[inline]
    pub fn read_i16(&mut self, bits: u64) -> Result<i16> {
        Ok(i16::from_ne_bytes(self.read_u16(bits)?.to_ne_bytes()))
    }

    /// Reads an unsigned 16bit integer with the given number of bits.
    #[inline]
    pub fn read_u16(&mut self, bits: u64) -> Result<u16> {
        if bits > u16::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        Ok(self.read_u64(bits)? as u16)
    }

    /// Reads a signed 32bit integer with the given number of bits.
    #[inline]
    pub fn read_i32(&mut self, bits: u64) -> Result<i32> {
        Ok(i32::from_ne_bytes(self.read_u32(bits)?.to_ne_bytes()))
    }

    /// Reads an unsigned 32bit integer with the given number of bits.
    #[inline]
    pub fn read_u32(&mut self, bits: u64) -> Result<u32> {
        if bits > u32::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        Ok(self.read_u64(bits)? as u32)
    }

    /// Reads a signed 64bit integer with the given number of bits.
    #[inline]
    pub fn read_i64(&mut self, bits: u64) -> Result<i64> {
        Ok(i64::from_ne_bytes(self.read_u64(bits)?.to_ne_bytes()))
    }

    /// Reads an unsigned 64bit integer with the given number of bits.
    #[inline]
    pub fn read_u64(&mut self, bits: u64) -> Result<u64> {
        if bits > u64::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        if self.msb {
            self.read_u64_msb(bits)
        } else {
            self.read_u64_lsb(bits)
        }
    }

    /// Reads an unsigned 64bit integer with the given number of bits in lsb order.
    #[inline]
    fn read_u64_lsb(&mut self, bits: u64) -> Result<u64> {
        let mut result = 0;
        let mut wei = 1;

        for _ in 0..bits {
            if self.buffer_offset >= self.buffer.len() * 8 {
                self.fill_buffer()?;
            }

            if self.buffer.is_empty() {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }

            if ((self.buffer[self.buffer_offset / 8] >> (self.buffer_offset % 8)) & 0x1) != 0 {
                result += wei;
            }

            wei *= 2;

            self.buffer_offset += 1;
        }

        Ok(result)
    }

    /// Reads an unsigned 64bit integer with the given number of bits in msb order.
    #[inline]
    fn read_u64_msb(&mut self, bits: u64) -> Result<u64> {
        let mut result = 0;

        for _ in 0..bits {
            if self.buffer_offset >= self.buffer.len() * 8 {
                self.fill_buffer()?;
            }

            if self.buffer.is_empty() {
                return Err(Error::from(ErrorKind::UnexpectedEof));
            }

            result <<= 1;

            if ((self.buffer[self.buffer_offset / 8] >> (7 - (self.buffer_offset % 8))) & 0x1) != 0
            {
                result |= 1;
            }

            self.buffer_offset += 1;
        }

        Ok(result)
    }

    /// Reads a 32bit float in unorm format with the given number of bits.
    /// `f32 = result as f32 / (1 << bits) - 1 as f32`
    #[inline]
    pub fn read_f32_unorm(&mut self, bits: u64) -> Result<f32> {
        let result = self.read_u64(bits)?;

        Ok(result as f32 / ((1 << bits) - 1) as f32)
    }

    /// Reads a 64bit float in unorm format with the given number of bits.
    /// `f64 = result as f64 / (1 << bits) - 1 as f64`
    #[inline]
    pub fn read_f64_unorm(&mut self, bits: u64) -> Result<f64> {
        let result = self.read_u64(bits)?;

        Ok(result as f64 / ((1 << bits) - 1) as f64)
    }

    /// Copys the entire contents of a bit stream to the bit sink.
    ///
    /// On success, the total number of bits copied is returned.
    #[inline]
    pub fn copy<O: Write>(&mut self, sink: &mut BitSink<O>) -> Result<u64> {
        let mut bits_copied = 0;

        loop {
            match self.read_bool() {
                Ok(value) => {
                    sink.write_bool(value)?;
                    bits_copied += 1;
                }
                Err(error) => {
                    if matches!(error.kind(), ErrorKind::UnexpectedEof) {
                        return Ok(bits_copied);
                    } else {
                        return Err(error);
                    }
                }
            }
        }
    }

    /// Fills the buffer with new bytes and resets the bit offset in the buffer.
    #[inline]
    fn fill_buffer(&mut self) -> Result<()> {
        self.buffer.resize(8, 0);

        let read = self.source.read(&mut self.buffer)?;

        self.buffer.resize(read, 0);
        self.buffer_offset = 0;

        Ok(())
    }
}

impl<T> BitStream<T>
where
    T: Read + Seek,
{
    /// Returns the current seek position, in bits, from the start of the stream.
    pub fn stream_position(&mut self) -> Result<u64> {
        let position = self.source.stream_position()?;

        if position == 0 || self.buffer.is_empty() {
            return Ok(0);
        }

        let byte_position = position - self.buffer.len() as u64;

        Ok(byte_position * 8 + self.buffer_offset as u64)
    }

    /// Rewind to the beginning of a stream.
    ///
    /// This is a convenience method, equivalent to `seek(SeekFrom::Start(0))`.
    pub fn rewind(&mut self) -> Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    /// Seek to an offset, in bits, in a stream.
    ///
    /// A seek beyond the end of a stream is allowed, but behavior is defined by the implementation.
    ///
    /// If the seek operation completed successfully, this method returns the new position from the start of the stream.
    /// That position can be used later with SeekFrom::Start.
    pub fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Current(current) => {
                let mut byte_position = self.source.seek(SeekFrom::Current(current / 8))?;

                if current < 0 && (current % 8) > 0 {
                    byte_position = self.source.seek(SeekFrom::Current(-1))?;

                    self.fill_buffer()?;
                    self.buffer_offset = (current % 8) as usize;
                } else {
                    self.fill_buffer()?;
                    self.buffer_offset = (current % 8) as usize;
                }

                Ok(byte_position * 8 + self.buffer_offset as u64)
            }
            SeekFrom::End(end) => {
                let mut byte_position = self.source.seek(SeekFrom::End(end / 8))?;

                if end < 0 && (end % 8) > 0 {
                    byte_position = self.source.seek(SeekFrom::Current(-1))?;

                    self.fill_buffer()?;
                    self.buffer_offset = (end % 8) as usize;
                } else {
                    self.fill_buffer()?;
                    self.buffer_offset = (end % 8) as usize;
                }

                Ok(byte_position * 8 + self.buffer_offset as u64)
            }
            SeekFrom::Start(start) => {
                self.source.seek(SeekFrom::Start(start / 8))?;
                self.fill_buffer()?;
                self.buffer_offset = (start % 8) as usize;

                Ok(start)
            }
        }
    }
}
