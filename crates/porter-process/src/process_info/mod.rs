#[cfg(target_os = "linux")]
mod process_info_linux;
#[cfg(target_os = "macos")]
mod process_info_macos;
#[cfg(target_os = "windows")]
mod process_info_win32;

use std::path::PathBuf;
use std::time::SystemTime;

/// Information about a process running on the local system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessInfo {
    /// The unique id of the process.
    pub pid: u64,
    /// The name of the process main module, without an extension.
    pub name: String,
    /// The path where the process executable module exists on disk.
    pub path: Option<PathBuf>,
    /// The time when the process was started.
    pub started_at: SystemTime,
}
