#[derive(Debug)]
pub enum ModelError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for ModelError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
