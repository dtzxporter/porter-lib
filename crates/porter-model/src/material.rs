use std::collections::HashSet;
use std::fmt;

use porter_utils::SanitizeFilename;

/// A material texture usage.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialTextureRefUsage {
    Unknown,
    Albedo,
    Diffuse,
    Specular,
    Normal,
    Emissive,
    Gloss,
    Roughness,
    AmbientOcclusion,
    Anisotropy,
    Cavity,
    Metalness,
    Count,
}

/// A texture reference for a material.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MaterialTextureRef {
    /// The file name for this texture, a relative path.
    pub file_name: String,
    /// The usage that this texture provides.
    pub texture_usage: MaterialTextureRefUsage,
    /// Tool specific alias that is unique to this texture.
    pub texture_alias: String,
    /// Tool specific hash that is unique to this texture.
    pub texture_hash: u64,
}

/// The parameter type.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialParameterType {
    BaseColor,
    Custom(String),
}

/// A parameter for a material.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialParameter {
    /// The parameter type or custom name.
    pub param: MaterialParameterType,
    /// The value for this parameter.
    pub value: MaterialParameterValue,
}

/// A parameter value for a material.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialParameterValue {
    /// A custom string value.
    String(String),
    /// A RGBA 32bit float color value.
    Color { r: f32, g: f32, b: f32, a: f32 },
}

/// A material which has a name, and is a collection of textures.
#[derive(Debug, Clone)]
pub struct Material {
    /// The sanitized name for this material.
    pub name: String,
    /// Used to differentiate between materials when remapping models.
    pub source_name: String,
    /// A collection of texture references for this material.
    pub textures: Vec<MaterialTextureRef>,
    /// A collection of parameters for this material.
    pub parameters: Vec<MaterialParameter>,
}

/// Cleans a material name.
fn sanitize_material_name(name: &str) -> String {
    let mut name = name.replace(' ', "_").sanitized();

    if name == "default" || name.is_empty() {
        name = String::from("_default");
    } else if name.as_bytes()[0].is_ascii_digit() {
        name = format!("_{}", name);
    }

    name
}

impl Material {
    /// Constructs a new material instance.
    pub fn new<N: Into<String>>(name: N) -> Self {
        let name = name.into();

        Self {
            name: sanitize_material_name(&name),
            source_name: name,
            textures: Vec::with_capacity(16),
            parameters: Vec::new(),
        }
    }

    /// Constructs a new material instance with a specific source name.
    pub fn with_source_name<N: Into<String>>(name: N, source_name: String) -> Self {
        let mut result = Self::new(name);

        result.source_name = source_name;
        result
    }

    /// Adds a texture to the material.
    pub fn push(&mut self, texture_ref: MaterialTextureRef) {
        self.textures.push(texture_ref);
    }

    /// Adds a parameter to the material.
    pub fn push_parameter<N: Into<MaterialParameterType>, P: Into<MaterialParameterValue>>(
        &mut self,
        param: N,
        value: P,
    ) {
        self.parameters.push(MaterialParameter {
            param: param.into(),
            value: value.into(),
        });
    }

    /// Removes the texture at the given index.
    pub fn remove(&mut self, index: usize) -> MaterialTextureRef {
        self.textures.remove(index)
    }

    /// Returns the number of textures in the material.
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// Whether or not the material is empty.
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }

    /// Returns a collection of unique textures that belong to this material.
    pub fn unique_textures(&self) -> HashSet<MaterialTextureRef> {
        self.textures
            .iter()
            .take(self.len())
            .filter(|x| !x.is_empty())
            .cloned()
            .collect::<HashSet<MaterialTextureRef>>()
    }

    /// Attempts to find the 'base' color parameter in this material.
    pub fn base_color(&self) -> Option<(f32, f32, f32, f32)> {
        self.parameters
            .iter()
            .find(|x| x.param == MaterialParameterType::BaseColor)
            .and_then(|x| {
                if let MaterialParameterValue::Color { r, g, b, a } = x.value {
                    Some((r, g, b, a))
                } else {
                    None
                }
            })
    }

    /// Attempts to find the 'base' color texture in this material.
    pub fn base_color_texture(&self) -> Option<&MaterialTextureRef> {
        self.textures
            .iter()
            .find(|x| x.texture_usage == MaterialTextureRefUsage::Albedo)
            .or_else(|| {
                self.textures
                    .iter()
                    .find(|x| x.texture_usage == MaterialTextureRefUsage::Diffuse)
            })
    }
}

impl MaterialTextureRef {
    /// Creates a new texture reference with an alias.
    #[inline]
    pub fn new<F: Into<String>, A: Into<String>>(
        file_name: F,
        usage: MaterialTextureRefUsage,
        alias: A,
    ) -> Self {
        debug_assert!(!matches!(usage, MaterialTextureRefUsage::Count));

        Self {
            file_name: file_name.into(),
            texture_usage: usage,
            texture_alias: alias.into(),
            texture_hash: 0,
        }
    }

    /// Creates a new texture reference with a hash.
    #[inline]
    pub fn with_hash<F: Into<String>>(
        file_name: F,
        usage: MaterialTextureRefUsage,
        hash: u64,
    ) -> Self {
        debug_assert!(!matches!(usage, MaterialTextureRefUsage::Count));

        Self {
            file_name: file_name.into(),
            texture_usage: usage,
            texture_alias: String::new(),
            texture_hash: hash,
        }
    }

    /// Whether or not the texture is empty or (default).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.texture_alias.is_empty() && self.texture_hash == 0
    }
}

impl Default for MaterialTextureRefUsage {
    #[inline]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for MaterialTextureRefUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaterialTextureRefUsage::Unknown => write!(f, "Unknown"),
            MaterialTextureRefUsage::Albedo => write!(f, "Albedo"),
            MaterialTextureRefUsage::Diffuse => write!(f, "Diffuse"),
            MaterialTextureRefUsage::Specular => write!(f, "Specular"),
            MaterialTextureRefUsage::Normal => write!(f, "Normal"),
            MaterialTextureRefUsage::Emissive => write!(f, "Emissive"),
            MaterialTextureRefUsage::Gloss => write!(f, "Gloss"),
            MaterialTextureRefUsage::Roughness => write!(f, "Roughness"),
            MaterialTextureRefUsage::AmbientOcclusion => write!(f, "Ambient Occlusion"),
            MaterialTextureRefUsage::Anisotropy => write!(f, "Anisotropy"),
            MaterialTextureRefUsage::Cavity => write!(f, "Cavity"),
            MaterialTextureRefUsage::Metalness => write!(f, "Metalness"),
            MaterialTextureRefUsage::Count => write!(f, "Count"),
        }
    }
}

impl From<String> for MaterialParameterType {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}

impl From<&str> for MaterialParameterType {
    fn from(value: &str) -> Self {
        Self::Custom(value.to_owned())
    }
}

impl From<String> for MaterialParameterValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for MaterialParameterValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<(f32, f32, f32, f32)> for MaterialParameterValue {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::Color {
            r: value.0,
            g: value.1,
            b: value.2,
            a: value.3,
        }
    }
}
