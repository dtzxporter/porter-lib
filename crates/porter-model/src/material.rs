use std::collections::HashSet;

use porter_utils::SanitizeExt;

use crate::MaterialTexture;
use crate::MaterialUsage;

/// The parameter type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MaterialParameterType {
    Usage(MaterialUsage),
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
    /// A Linear RGBA 32bit float color value.
    ColorLinear { r: f32, g: f32, b: f32, a: f32 },
    /// A sRGB RGBA 32bit float color value.
    ColorSRGB { r: f32, g: f32, b: f32, a: f32 },
}

/// A material which has a name, and is a collection of textures.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    /// The sanitized name for this material.
    pub name: String,
    /// Used to differentiate between materials when remapping models.
    pub source_name: String,
    /// A collection of texture references for this material.
    pub textures: Vec<MaterialTexture>,
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
        let name: String = name.into();

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
    pub fn push(&mut self, texture: MaterialTexture) {
        let indices = self
            .textures
            .binary_search_by_key(&texture.usage, |entry| entry.usage);

        match indices {
            Ok(index) => self.textures.insert(index + 1, texture),
            Err(index) => self.textures.insert(index, texture),
        }
    }

    /// Adds a parameter to the material.
    pub fn push_parameter<N: Into<MaterialParameterType>, P: Into<MaterialParameterValue>>(
        &mut self,
        param: N,
        value: P,
    ) {
        let param = param.into();
        let value = value.into();

        let indices = self
            .parameters
            .binary_search_by_key(&param, |entry| entry.param);

        match indices {
            Ok(index) => self
                .parameters
                .insert(index + 1, MaterialParameter { param, value }),
            Err(index) => self
                .parameters
                .insert(index, MaterialParameter { param, value }),
        }
    }

    /// Removes the texture at the given index.
    pub fn remove(&mut self, index: usize) -> MaterialTexture {
        self.textures.remove(index)
    }

    /// Extend this material with the contents of another.
    pub fn extend(&mut self, other: Self) {
        for texture in other.textures {
            self.push(texture);
        }

        for parameter in other.parameters {
            self.push_parameter(parameter.param, parameter.value);
        }
    }

    /// Returns a collection of unique textures that belong to this material.
    pub fn unique_textures(&self) -> HashSet<MaterialTexture> {
        self.textures
            .iter()
            .filter(|x| !x.is_empty())
            .cloned()
            .collect::<HashSet<MaterialTexture>>()
    }

    /// Attempts to find the 'base' color texture in this material.
    pub fn base_color_texture(&self) -> Option<&MaterialTexture> {
        self.textures
            .iter()
            .find(|x| x.usage == MaterialUsage::Albedo)
            .or_else(|| {
                self.textures
                    .iter()
                    .find(|x| x.usage == MaterialUsage::Diffuse)
            })
    }
}

impl From<MaterialUsage> for MaterialParameterType {
    fn from(value: MaterialUsage) -> Self {
        Self::Usage(value)
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
