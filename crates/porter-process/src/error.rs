/// Errors that can occur in the process crate.
#[derive(Debug)]
pub enum ProcessError {
    NotFound,
    AccessDenied,
    IoError(std::io::Error),
    TryReserveError(std::collections::TryReserveError),
    #[cfg(target_os = "linux")]
    ProcError(procfs::ProcError),
}

impl From<std::io::Error> for ProcessError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::collections::TryReserveError> for ProcessError {
    fn from(value: std::collections::TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}

#[cfg(target_os = "linux")]
impl From<procfs::ProcError> for ProcessError {
    fn from(value: procfs::ProcError) -> Self {
        Self::ProcError(value)
    }
}

impl From<ProcessError> for std::io::Error {
    fn from(value: ProcessError) -> Self {
        match value {
            ProcessError::IoError(error) => error,
            _ => Self::last_os_error(),
        }
    }
}
