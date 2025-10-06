use std::io;
use std::io::SeekFrom;
use std::sync::Arc;

use porter_utils::SeekExt;

use crate::ProcessError;
use crate::ProcessHandle;
use crate::ProcessHandlePlatform;

/// An open process for reading.
#[derive(Debug, Clone)]
pub struct ProcessReader {
    offset: u64,
    handle: Arc<ProcessHandle>,
}

impl ProcessReader {
    /// Constructs a new process reader from the given handle.
    pub(crate) fn from_handle(handle: Arc<ProcessHandle>) -> Self {
        Self { offset: 0, handle }
    }

    /// Gets the base address from the process.
    pub fn base_address(&self) -> Result<u64, ProcessError> {
        self.handle.base_address()
    }

    /// Gets the size of the main module in bytes.
    pub fn main_module_size(&self) -> Result<u64, ProcessError> {
        self.handle.main_module_size()
    }

    /// Creates a new copy of this process reader for reading at the given offset.
    pub fn reader_at<P: Copy + 'static>(&self, offset: P) -> Result<Self, ProcessError>
    where
        u64: TryFrom<P>,
    {
        let mut reader = self.clone();

        reader.reset_to(offset)?;

        Ok(reader)
    }
}

impl io::Read for ProcessReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.handle.read(self.offset, buf)?;

        self.offset += read as u64;

        Ok(read)
    }
}

impl io::Seek for ProcessReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Current(offset) => {
                self.offset = (self.offset as i64).wrapping_add(offset) as _;
            }
            SeekFrom::End(offset) => {
                self.offset = (i64::MAX).wrapping_add(offset) as _;
            }
            SeekFrom::Start(offset) => {
                self.offset = offset;
            }
        }

        Ok(self.offset)
    }
}
