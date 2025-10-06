/// Errors that can occur in the preview crate.
#[derive(Debug)]
pub enum PreviewError {
    Unsupported,
    InvalidAsset,
    OutOfMemory,
    IoError(std::io::Error),
    TryReserveError(std::collections::TryReserveError),
}

impl From<std::io::Error> for PreviewError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::collections::TryReserveError> for PreviewError {
    fn from(value: std::collections::TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}
