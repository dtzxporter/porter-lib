#[derive(Default, Clone, Copy)]
pub enum ImageConvertOptions {
    /// Do nothing to modify the image.
    #[default]
    None,
    /// Always reconstruct the Z channel of the image.
    ReconstructZ,
    /// Always reconstruct the Z channel and invert the Y channel of the image.
    ReconstructZInvertY,
    /// Only reconstruct the Z channel of the image when the format is Bc5Unorm.
    AutoReconstructZ,
    /// Only reconstruct the Z channel and invert the Y channel of the image when the format is Bc5Unorm.
    AutoReconstructZInvertY,
    /// Transform the image by scale and bias.
    UniformScaleBias(f32, f32),
}
