use bincode::Decode;
use bincode::Encode;

/// Represents a supported animation file type.
#[derive(Decode, Encode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationFileType {
    SEAnim,
    Cast,
}
