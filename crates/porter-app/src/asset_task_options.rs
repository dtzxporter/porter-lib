use std::path::PathBuf;

/// Options for configuring the image export task.
pub struct AssetTaskImageOptions {
    pub(crate) override_path: Option<PathBuf>,
    pub(crate) name_override: Option<String>,
    pub(crate) normal_map_mode: AssetTaskImageNormalMapMode,
    pub(crate) normal_map_transform: bool,
    pub(crate) append_extension: bool,
}

/// Normal map processing for an image export task.
#[derive(Debug, Clone, Copy)]
pub enum AssetTaskImageNormalMapMode {
    /// Do not do any normal map processing at all.
    Ignored,
    /// Image may be an OpenGl normal map and should be converted if Bc5.
    OpenGlAuto,
    /// Image is always an OpenGl normal map and should always be converted.
    OpenGlForced,
    /// Image may be a DirectX normal map and should be converted if Bc5.
    DirectXAuto,
    /// Image is a DirectX normal map and should always be converted.
    DirectXForced,
}

impl AssetTaskImageOptions {
    /// Constructs new image task options.
    pub const fn new() -> Self {
        Self {
            override_path: None,
            name_override: None,
            normal_map_mode: AssetTaskImageNormalMapMode::Ignored,
            normal_map_transform: false,
            append_extension: false,
        }
    }

    /// Sets a custom override path.
    pub fn with_override_path(mut self, path: Option<PathBuf>) -> Self {
        self.override_path = path;
        self
    }

    /// Sets the name override.
    pub fn with_name_override(mut self, name: Option<String>) -> Self {
        self.name_override = name;
        self
    }

    /// Sets the normal map mode.
    pub const fn with_normal_map_mode(mut self, mode: AssetTaskImageNormalMapMode) -> Self {
        self.normal_map_mode = mode;
        self
    }

    /// Applies the normal map mode using a transform before conversion.
    pub const fn with_normal_map_transform(mut self) -> Self {
        self.normal_map_transform = true;
        self
    }

    /// Appends the image file extension instead of replacing an existing one.
    pub const fn with_append_extension(mut self) -> Self {
        self.append_extension = true;
        self
    }
}

impl Default for AssetTaskImageOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for configuring the model export task.
pub struct AssetTaskModelOptions {
    pub(crate) output_path: PathBuf,
    pub(crate) name_override: Option<String>,
    pub(crate) scale_default: f32,
}

impl AssetTaskModelOptions {
    /// Constructs new model task options.
    pub fn new() -> Self {
        Self {
            output_path: PathBuf::new(),
            name_override: None,
            scale_default: 1.0,
        }
    }

    /// Sets the output path.
    pub fn with_output_path(mut self, path: PathBuf) -> Self {
        self.output_path = path;
        self
    }

    /// Sets the name override.
    pub fn with_name_override(mut self, name: Option<String>) -> Self {
        self.name_override = name;
        self
    }

    /// Sets the default scale factor.
    pub const fn with_scale_default(mut self, default: f32) -> Self {
        self.scale_default = default;
        self
    }
}

impl Default for AssetTaskModelOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for configuring the material export task and material paths.
pub struct AssetTaskMaterialOptions {
    pub(crate) override_path: Option<PathBuf>,
    pub(crate) extract: bool,
}

impl AssetTaskMaterialOptions {
    /// Constructs new material task options.
    pub const fn new() -> Self {
        Self {
            override_path: None,
            extract: false,
        }
    }

    /// Sets a custom override path.
    pub fn with_override_path(mut self, path: Option<PathBuf>) -> Self {
        self.override_path = path;
        self
    }

    /// Sets whether or not to extract material images.
    pub const fn with_extract(mut self, extract: bool) -> Self {
        self.extract = extract;
        self
    }
}

impl Default for AssetTaskMaterialOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for configuring the animation export task.
pub struct AssetTaskAnimationOptions {
    pub(crate) override_path: Option<PathBuf>,
    pub(crate) name_override: Option<String>,
    pub(crate) scale_default: f32,
}

impl AssetTaskAnimationOptions {
    /// Constructs new animation task options.
    pub const fn new() -> Self {
        Self {
            override_path: None,
            name_override: None,
            scale_default: 1.0,
        }
    }

    /// Sets a custom override path.
    pub fn with_override_path(mut self, path: Option<PathBuf>) -> Self {
        self.override_path = path;
        self
    }

    /// Sets the name override.
    pub fn with_name_override(mut self, name: Option<String>) -> Self {
        self.name_override = name;
        self
    }

    /// Sets the default scale factor.
    pub const fn with_scale_default(mut self, default: f32) -> Self {
        self.scale_default = default;
        self
    }
}

impl Default for AssetTaskAnimationOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for configuring the world export task.
pub struct AssetTaskWorldOptions {
    pub(crate) name_suffix: Option<String>,
    pub(crate) scale_default: f32,
}

impl AssetTaskWorldOptions {
    /// Constructs new world task options.
    pub const fn new() -> Self {
        Self {
            name_suffix: None,
            scale_default: 1.0,
        }
    }

    /// Sets a custom name suffix.
    pub fn with_name_suffix(mut self, suffix: Option<String>) -> Self {
        self.name_suffix = suffix;
        self
    }

    /// Sets the default scale factor.
    pub const fn with_scale_default(mut self, default: f32) -> Self {
        self.scale_default = default;
        self
    }
}

impl Default for AssetTaskWorldOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for configuring the raw file export task.
pub struct AssetTaskRawFileOptions<'a> {
    pub(crate) custom_name: &'a str,
    pub(crate) create_directory: bool,
}

impl<'a> AssetTaskRawFileOptions<'a> {
    /// Constructs new raw file task options.
    pub const fn new() -> Self {
        Self {
            custom_name: "raw_files",
            create_directory: true,
        }
    }

    /// Sets the custom name.
    pub const fn with_custom_name(mut self, name: &'a str) -> Self {
        self.custom_name = name;
        self
    }

    /// Sets whether or not to create the directory.
    pub const fn with_create_directory(mut self, create: bool) -> Self {
        self.create_directory = create;
        self
    }
}

impl<'a> Default for AssetTaskRawFileOptions<'a> {
    fn default() -> Self {
        Self::new()
    }
}
