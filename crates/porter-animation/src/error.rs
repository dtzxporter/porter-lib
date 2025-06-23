/// Errors that can occur in the animation crate.
#[derive(Debug)]
pub enum AnimationError {
    IoError(std::io::Error),
    CurveAllocationFailed,
}

impl From<std::io::Error> for AnimationError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
