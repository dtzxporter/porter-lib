use porter_model::MaterialTextureRefUsage;
use porter_model::Model;

use porter_texture::Image;

/// An asset which is ready to be previewed.
#[derive(Debug, Clone)]
pub enum PorterPreviewAsset {
    /// An image asset for preview.
    Image(String, Image),
    /// A model asset for preview.
    Model(String, Model, Vec<Option<Image>>),
    /// A material asset for preview.
    Material(String, Vec<(MaterialTextureRefUsage, Image)>),
}
