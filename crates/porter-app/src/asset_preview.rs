use porter_model::MaterialUsage;
use porter_model::Model;

use porter_texture::Image;

use porter_audio::Audio;

/// The result of an assets data to be previewed.
#[derive(Debug)]
pub enum AssetPreview {
    /// This asset type doesn't support preview.
    NotSupported,
    /// An error occured while previewing this asset.
    PreviewError,
    /// A raw file asset for preview.
    RawFile(String, Vec<u8>),
    /// An image asset for preview.
    Image(String, Image),
    /// A model asset for preview.
    Model(String, Model, Vec<Option<Image>>),
    /// A material asset for preview.
    Material(String, Vec<(MaterialUsage, Image)>),
    /// A audio asset for preview.
    Audio(String, Audio),
}

impl Clone for AssetPreview {
    #[inline]
    fn clone(&self) -> Self {
        // We do not want asset preview to be cloned, but it has to implement Clone
        // in order to be passed in a message from iced.
        Self::NotSupported
    }
}
