#[derive(Debug)]
pub enum AnimationError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for AnimationError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
