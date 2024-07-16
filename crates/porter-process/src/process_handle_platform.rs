use crate::ProcessError;

/// Shared platform process handle trait.
pub trait ProcessHandlePlatform
where
    Self: Sized,
{
    /// Opens the process with the given pid, read and write flags.
    fn open_process(pid: u64, read: bool, write: bool) -> Result<Self, ProcessError>;
    /// Reads a block of memory from the process at the given offset.
    fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize, ProcessError>;
    /// Gets the base address of the process.
    fn base_address(&self) -> Result<u64, ProcessError>;
    /// Gets the size of the main module in bytes.
    fn main_module_size(&self) -> Result<u64, ProcessError>;
    /// Closes the handle of the process.
    fn close(&mut self);
}
