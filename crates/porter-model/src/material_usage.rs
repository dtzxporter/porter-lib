use std::fmt;

/// A material usage.
#[repr(u32)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MaterialUsage {
    Albedo,
    Diffuse,
    Specular,
    Normal,
    Emissive,
    EmissiveMask,
    EmissiveStrength,
    Gloss,
    Roughness,
    AmbientOcclusion,
    Anisotropy,
    Cavity,
    Metalness,
    #[default]
    Unknown,
    Count,
}

impl fmt::Display for MaterialUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaterialUsage::Unknown => write!(f, "Unknown"),
            MaterialUsage::Albedo => write!(f, "Albedo"),
            MaterialUsage::Diffuse => write!(f, "Diffuse"),
            MaterialUsage::Specular => write!(f, "Specular"),
            MaterialUsage::Normal => write!(f, "Normal"),
            MaterialUsage::Emissive => write!(f, "Emissive"),
            MaterialUsage::EmissiveMask => write!(f, "Emissive Mask"),
            MaterialUsage::EmissiveStrength => write!(f, "Emissive Strength"),
            MaterialUsage::Gloss => write!(f, "Gloss"),
            MaterialUsage::Roughness => write!(f, "Roughness"),
            MaterialUsage::AmbientOcclusion => write!(f, "Ambient Occlusion"),
            MaterialUsage::Anisotropy => write!(f, "Anisotropy"),
            MaterialUsage::Cavity => write!(f, "Cavity"),
            MaterialUsage::Metalness => write!(f, "Metalness"),
            MaterialUsage::Count => write!(f, "Count"),
        }
    }
}
