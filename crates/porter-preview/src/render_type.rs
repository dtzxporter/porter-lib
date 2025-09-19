use crate::RenderImage;
use crate::RenderMaterial;
use crate::RenderModel;

/// The type of 3d render asset to preview.
pub enum RenderType {
    Model(RenderModel),
    Image(RenderImage),
    Material(RenderMaterial),
}
