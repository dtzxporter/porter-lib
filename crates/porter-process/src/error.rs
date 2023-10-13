#[derive(Debug)]
pub enum ProcessError {
    NotFound,
    AccessDenied,
    IoError(std::io::Error),
    #[cfg(target_os = "windows")]
    NulErrorU16(widestring::error::NulError<u16>),
    #[cfg(target_os = "linux")]
    ProcError(procfs::ProcError),
}

impl From<std::io::Error> for ProcessError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[cfg(target_os = "windows")]
impl From<widestring::error::NulError<u16>> for ProcessError {
    fn from(value: widestring::error::NulError<u16>) -> Self {
        Self::NulErrorU16(value)
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
