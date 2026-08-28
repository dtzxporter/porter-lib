/// Errors that can occur in asset tasks.
#[derive(Debug)]
pub enum AssetTaskError {
    FileNameMissing,
    InvalidOption,
    IoError(std::io::Error),
    ModelError(porter_model::ModelError),
    AudioError(porter_audio::AudioError),
    TextureError(porter_texture::TextureError),
    AnimationError(porter_animation::AnimationError),
}

impl From<std::io::Error> for AssetTaskError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<porter_model::ModelError> for AssetTaskError {
    fn from(value: porter_model::ModelError) -> Self {
        Self::ModelError(value)
    }
}

impl From<porter_audio::AudioError> for AssetTaskError {
    fn from(value: porter_audio::AudioError) -> Self {
        Self::AudioError(value)
    }
}

impl From<porter_texture::TextureError> for AssetTaskError {
    fn from(value: porter_texture::TextureError) -> Self {
        Self::TextureError(value)
    }
}

impl From<porter_animation::AnimationError> for AssetTaskError {
    fn from(value: porter_animation::AnimationError) -> Self {
        Self::AnimationError(value)
    }
}
