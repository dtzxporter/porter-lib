/// Specifies how texture coordinates resolve when sampling an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    /// Texture coordinates are clamped to `[0.0, 1.0]`.
    Clamp,
    /// Texture coordinates wrap around.
    Wrap,
}
