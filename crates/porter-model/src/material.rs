use std::collections::HashSet;

use porter_utils::SanitizeFilename;

/// A material texture usage.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialTextureRefUsage {
    Unknown = 0,
    Albedo = 1,
    Diffuse = 2,
    Specular = 3,
    Normal = 4,
    Emissive = 5,
    Gloss = 6,
    Roughness = 7,
    AmbientOcclusion = 8,
    Anisotropy = 9,
    Cavity = 10,
}

/// A texture reference for a material.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MaterialTextureRef {
    pub file_name: String,
    pub texture_usage: MaterialTextureRefUsage,
    pub texture_alias: String,
    pub texture_hash: u64,
}

/// The maximum number of textures in a material.
pub const MAXIMUM_MATERIAL_TEXTURES: usize = 0x10;

/// A material which has a name, and is a collection of textures.
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub textures: [MaterialTextureRef; MAXIMUM_MATERIAL_TEXTURES],
    textures_count: usize,
}

/// Cleans a material name.
pub(crate) fn sanitize_material_name(name: String) -> String {
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
    #[inline]
    pub fn new<N: Into<String>>(name: N) -> Self {
        Self {
            name: sanitize_material_name(name.into()),
            textures: [
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
                MaterialTextureRef::default(),
            ],
            textures_count: 0,
        }
    }

    /// Adds a texture to the material.
    pub fn push(&mut self, texture_ref: MaterialTextureRef) {
        debug_assert!(self.textures_count != self.textures.len());

        self.textures[self.textures_count] = texture_ref;
        self.textures_count += 1;
    }

    /// Removes the texture at the given index.
    pub fn remove(&mut self, index: usize) -> MaterialTextureRef {
        debug_assert!(index < self.textures_count);

        let swapped = std::mem::take(&mut self.textures[index]);

        // We need to shift values from right of index to the left.
        // if it's the last value, we can ignore it, already default.
        for i in index..(self.textures_count.max(1) - 1) {
            self.textures.swap(i + 1, i);
        }

        self.textures_count -= 1;

        swapped
    }

    /// Returns the number of textures in the material.
    pub fn len(&self) -> usize {
        self.textures_count
    }

    /// Whether or not the material is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

    /// Attempts to find the 'base' color texture in this material.
    pub fn base_texture(&self) -> Option<&MaterialTextureRef> {
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

impl ToString for MaterialTextureRefUsage {
    fn to_string(&self) -> String {
        match self {
            MaterialTextureRefUsage::Unknown => String::from("Unknown"),
            MaterialTextureRefUsage::Albedo => String::from("Albedo"),
            MaterialTextureRefUsage::Diffuse => String::from("Diffuse"),
            MaterialTextureRefUsage::Specular => String::from("Specular"),
            MaterialTextureRefUsage::Normal => String::from("Normal"),
            MaterialTextureRefUsage::Emissive => String::from("Emissive"),
            MaterialTextureRefUsage::Gloss => String::from("Gloss"),
            MaterialTextureRefUsage::Roughness => String::from("Roughness"),
            MaterialTextureRefUsage::AmbientOcclusion => String::from("Ambient Occlusion"),
            MaterialTextureRefUsage::Anisotropy => String::from("Anisotropy"),
            MaterialTextureRefUsage::Cavity => String::from("Cavity"),
        }
    }
}
