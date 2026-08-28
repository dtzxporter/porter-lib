/// Errors that can occur in the viewport crate.
#[derive(Debug)]
pub enum ViewportError {
    Unsupported,
    InvalidAsset,
    OutOfMemory,
    IoError(std::io::Error),
    TryReserveError(std::collections::TryReserveError),
}

impl From<std::io::Error> for ViewportError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::collections::TryReserveError> for ViewportError {
    fn from(value: std::collections::TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}
