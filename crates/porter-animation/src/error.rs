/// Errors that can occur in the animation crate.
#[derive(Debug)]
pub enum AnimationError {
    IoError(std::io::Error),
    TryReserveError(std::collections::TryReserveError),
    InvalidKeyframeValue,
    InvalidJointName,
}

impl From<std::io::Error> for AnimationError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::collections::TryReserveError> for AnimationError {
    fn from(value: std::collections::TryReserveError) -> Self {
        Self::TryReserveError(value)
    }
}
