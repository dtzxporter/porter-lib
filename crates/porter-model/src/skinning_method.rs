/// The skinning method to use when deforming a mesh.
#[derive(Debug, Clone, Copy)]
pub enum SkinningMethod {
    /// Linear, the default skinning method.
    Linear,
    /// Dual quaternion skinning method.
    DualQuaternion,
}
