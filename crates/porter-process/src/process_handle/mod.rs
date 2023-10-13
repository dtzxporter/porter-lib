#[cfg(target_os = "linux")]
mod process_handle_linux;
#[cfg(target_os = "macos")]
mod process_handle_macos;
#[cfg(target_os = "windows")]
mod process_handle_win32;

#[cfg(target_os = "windows")]
type Handle = windows_sys::Win32::Foundation::HANDLE;
#[cfg(target_os = "macos")]
type Handle = libc::mach_port_t;
#[cfg(target_os = "linux")]
type Handle = libc::pid_t;

use crate::ProcessHandlePlatform;

/// A handle to a process on the local system.
#[derive(Debug)]
pub struct ProcessHandle {
    /// Platform specific handle of the process.
    handle: Handle,
    /// Whether or not this handle can read from the process.
    can_read: bool,
    /// Whether or not this handle can write to the process.
    can_write: bool,
}

impl ProcessHandle {
    /// Returns true if the handle can be used to read from the process.
    pub fn can_read(&self) -> bool {
        self.can_read
    }

    /// Returns true if the handle can be used to write to the process.
    pub fn can_write(&self) -> bool {
        self.can_write
    }
}

impl Drop for ProcessHandle
where
    Self: ProcessHandlePlatform,
{
    fn drop(&mut self) {
        self.close();
    }
}
