use crate::MaterialUsage;

/// A texture reference for a material.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MaterialTexture {
    /// The file name for this texture, a relative path.
    pub file_path: String,
    /// The usage that this texture provides.
    pub usage: MaterialUsage,
    /// Tool specific alias that is unique to this texture.
    pub alias: String,
    /// Tool specific hash that is unique to this texture.
    pub hash: u64,
}

impl MaterialTexture {
    /// Creates a new material texture.
    pub fn new<F: Into<String>>(file_path: F) -> Self {
        Self {
            file_path: file_path.into(),
            usage: Default::default(),
            alias: String::new(),
            hash: 0,
        }
    }

    /// Sets the material usage for this texture.
    pub const fn with_usage(mut self, usage: MaterialUsage) -> Self {
        self.usage = usage;
        self
    }

    /// Sets the tool specific hash that is unique to this texture.
    pub const fn with_hash(mut self, hash: u64) -> Self {
        self.hash = hash;
        self
    }

    /// Sets the tool specific alias that is unique to this texture.
    pub fn with_alias<A: Into<String>>(mut self, alias: A) -> Self {
        self.alias = alias.into();
        self
    }

    /// Whether or not the texture is empty or (default).
    pub fn is_empty(&self) -> bool {
        self.alias.is_empty() && self.hash == 0
    }
}
