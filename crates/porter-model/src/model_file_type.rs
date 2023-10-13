use bincode::Decode;
use bincode::Encode;

/// Represents a supported model file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFileType {
    Obj,
    Smd,
    XnaLara,
    XModelExport,
    SEModel,
    Cast,
    Maya,
    Fbx,
}
