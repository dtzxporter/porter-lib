use std::io::Cursor;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::Write;

/// A bit sink that provides methods for writing data as bits (Lsb order).
pub struct BitSink<T> {
    output: T,
    buffer: u8,
    buffer_offset: usize,
    msb: bool,
}

impl BitSink<()> {
    /// Constructs a new bit sink using a growable memory buffer in lsb order.
    #[inline]
    pub const fn new_lsb() -> BitSink<Cursor<Vec<u8>>> {
        BitSink::with_output_lsb(Cursor::new(Vec::new()))
    }

    /// Constructs a new bit sink using a growable memory buffer with an initial capacity in bits, in lsb order.
    #[inline]
    pub fn with_capacity_lsb(capacity: usize) -> BitSink<Cursor<Vec<u8>>> {
        BitSink::with_output_lsb(Cursor::new(Vec::with_capacity((capacity + 7) / 8)))
    }

    /// Constructs a new bit sink using a growable memory buffer in msb order.
    #[inline]
    pub const fn new_msb() -> BitSink<Cursor<Vec<u8>>> {
        BitSink::with_output_msb(Cursor::new(Vec::new()))
    }

    /// Constructs a new bit sink using a growable memory buffer with an initial capacity in bits, in msb order.
    #[inline]
    pub fn with_capacity_msb(capacity: usize) -> BitSink<Cursor<Vec<u8>>> {
        BitSink::with_output_msb(Cursor::new(Vec::with_capacity((capacity + 7) / 8)))
    }
}

impl<T> BitSink<T> {
    /// Gets a mutable reference to the inner output.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.output
    }

    /// Gets a reference to the inner output.
    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.output
    }
}

impl<T> BitSink<T>
where
    T: Write,
{
    /// Constructs a new bit sink from the given output in lsb order.
    #[inline]
    pub const fn with_output_lsb(output: T) -> Self {
        Self {
            output,
            buffer: 0,
            buffer_offset: 0,
            msb: false,
        }
    }

    /// Constructs a new bit sink from the given output in msb order.
    #[inline]
    pub const fn with_output_msb(output: T) -> Self {
        Self {
            output,
            buffer: 0,
            buffer_offset: 0,
            msb: true,
        }
    }

    /// Aligns the bit sink to the next byte.
    #[inline]
    pub fn align(&mut self) -> Result<()> {
        if self.buffer_offset > 0 {
            self.flush_buffer()?;
            self.buffer_offset = 0;
        }

        Ok(())
    }

    /// Writes all of the bytes from the slice to the stream.
    #[inline]
    pub fn write_all<S: AsRef<[u8]>>(&mut self, slice: S) -> Result<()> {
        for byte in slice.as_ref() {
            self.write_u8(8, *byte)?;
        }

        Ok(())
    }

    /// Writes a 1bit boolean value.
    #[inline]
    pub fn write_bool(&mut self, value: bool) -> Result<()> {
        if self.buffer_offset == u8::BITS as usize {
            self.flush_buffer()?;
        }

        if value {
            if self.msb {
                self.buffer |= 1 << (7 - self.buffer_offset);
            } else {
                self.buffer |= 1 << self.buffer_offset;
            }
        }

        self.buffer_offset += 1;

        Ok(())
    }

    /// Writes a signed 8bit integer with the given number of bits.
    #[inline]
    pub fn write_i8(&mut self, bits: u64, value: i8) -> Result<()> {
        self.write_u8(bits, u8::from_ne_bytes(value.to_ne_bytes()))
    }

    /// Writes an unsigned 8bit integer with the given number of bits.
    #[inline]
    pub fn write_u8(&mut self, bits: u64, value: u8) -> Result<()> {
        if bits > u8::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        self.write_u64(bits, value as u64)
    }

    /// Writes a signed 16bit integer with the given number of bits.
    #[inline]
    pub fn write_i16(&mut self, bits: u64, value: i16) -> Result<()> {
        self.write_u16(bits, u16::from_ne_bytes(value.to_ne_bytes()))
    }

    /// Writes an unsigned 16bit integer with the given number of bits.
    #[inline]
    pub fn write_u16(&mut self, bits: u64, value: u16) -> Result<()> {
        if bits > u16::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        self.write_u64(bits, value as u64)
    }

    /// Writes a signed 32bit integer with the given number of bits.
    #[inline]
    pub fn write_i32(&mut self, bits: u64, value: i32) -> Result<()> {
        self.write_u32(bits, u32::from_ne_bytes(value.to_ne_bytes()))
    }

    /// Writes an unsigned 32bit integer with the given number of bits.
    #[inline]
    pub fn write_u32(&mut self, bits: u64, value: u32) -> Result<()> {
        if bits > u32::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        self.write_u64(bits, value as u64)
    }

    /// Writes a signed 64bit integer with the given number of bits.
    #[inline]
    pub fn write_i64(&mut self, bits: u64, value: i64) -> Result<()> {
        self.write_u64(bits, u64::from_ne_bytes(value.to_ne_bytes()))
    }

    /// Writes an unsigned 64bit integer with the given number of bits.
    #[inline]
    pub fn write_u64(&mut self, bits: u64, value: u64) -> Result<()> {
        if bits > u64::BITS as u64 {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        if self.msb {
            for bit in (0..bits).rev() {
                self.write_bool((value & (1 << bit)) > 0)?;
            }
        } else {
            for bit in 0..bits {
                self.write_bool((value & (1 << bit)) > 0)?;
            }
        }

        Ok(())
    }

    /// Writes a 32bit float in unorm format with the given number of bits.
    #[inline]
    pub fn write_f32_unorm(&mut self, bits: u64, value: f32) -> Result<()> {
        let max: u64 = (1 << bits) - 1;
        let value = value * max as f32;

        self.write_u64(bits, value.clamp(0.0, max as f32) as u64)
    }

    /// Writes a 64bit float in unorm format with the given number of bits.
    #[inline]
    pub fn write_f64_unorm(&mut self, bits: u64, value: f64) -> Result<()> {
        let max: u64 = (1 << bits) - 1;
        let value = value * max as f64;

        self.write_u64(bits, value.clamp(0.0, max as f64) as u64)
    }

    /// Flushes the buffer to output, and aligning to the next byte.
    #[inline]
    pub fn flush(&mut self) -> Result<()> {
        self.flush_buffer()
    }

    /// Consumes the bit sink and returns the inner output.
    ///
    /// The buffer is written out before returning the output.
    #[inline]
    pub fn into_inner(mut self) -> Result<T> {
        self.flush()?;

        Ok(self.output)
    }

    /// Flushes the current buffer to output, and resets the bit offset.
    #[inline]
    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer_offset > 0 {
            self.output.write_all(&[self.buffer])?;
        }

        self.buffer = 0;
        self.buffer_offset = 0;

        Ok(())
    }
}
