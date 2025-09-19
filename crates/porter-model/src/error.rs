/// Errors that can occur in the model crate.
#[derive(Debug)]
pub enum ModelError {
    IoError(std::io::Error),
    TryReserveError(std::collections::TryReserveError),
}

impl From<std::io::Error> for ModelError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::collections::TryReserveError> for ModelError {
    fn from(value: std::collections::TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}
