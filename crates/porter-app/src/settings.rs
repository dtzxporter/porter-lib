use std::mem::transmute;
use std::path::Path;
use std::path::PathBuf;

use miniserde::Deserialize;
use miniserde::Serialize;

use porter_animation::AnimationFileType;
use porter_audio::AudioFileType;
use porter_model::ModelFileType;
use porter_texture::ImageFileType;

use porter_utils::BitFlags;
use porter_utils::bitflags;

use crate::system;

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct LoadSettings(BitFlags<u32>);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct ModelSettings(BitFlags<u32>);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct AnimSettings(BitFlags<u32>);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct AudioSettings(BitFlags<u32>);

macro_rules! impl_miniserde {
    ($type:ty) => {
        impl Deserialize for $type {
            fn begin(out: &mut Option<Self>) -> &mut dyn ::miniserde::de::Visitor {
                // SAFETY: All settings are repr(transparent) so we can transmute to the base type.
                Deserialize::begin(unsafe { transmute::<&mut Option<Self>, &mut Option<u32>>(out) })
            }
        }

        impl Serialize for $type {
            fn begin(&self) -> ::miniserde::ser::Fragment<'_> {
                self.0.begin()
            }
        }
    };
}

impl_miniserde!(LoadSettings);
impl_miniserde!(ModelSettings);
impl_miniserde!(AnimSettings);
impl_miniserde!(AudioSettings);

bitflags! {
    impl LoadSettings: u32 {
        const LOAD_MODELS: u32 = 1 << 0;
        const LOAD_IMAGES: u32 = 1 << 1;
        const LOAD_MATERIALS: u32 = 1 << 2;
        const LOAD_ANIMATIONS: u32 = 1 << 3;
        const LOAD_SOUNDS: u32 = 1 << 4;
        const LOAD_RAW_FILES: u32 = 1 << 5;
        const LOAD_FORCE_RAW_FILES: u32 = 1 << 6;
    }
}

bitflags! {
    impl ModelSettings: u32 {
        const EXPORT_OBJ: u32 = 1 << 0;
        const EXPORT_SMD: u32 = 1 << 1;
        const EXPORT_XNA_LARA: u32 = 1 << 2;
        const EXPORT_XMODEL_EXPORT: u32 = 1 << 3;
        #[allow(unused)]
        const EXPORT_SEMODEL_REMOVED: u32 = 1 << 4;
        const EXPORT_CAST: u32 = 1 << 5;
        const EXPORT_MAYA: u32 = 1 << 6;
        const EXPORT_FBX: u32 = 1 << 7;
    }
}

bitflags! {
    impl AnimSettings: u32 {
        #[allow(unused)]
        const EXPORT_SEANIM_REMOVED: u32 = 1 << 0;
        const EXPORT_CAST: u32 = 1 << 1;
    }
}

bitflags! {
    impl AudioSettings: u32 {
        const EXPORT_WAV: u32 = 1 << 0;
        const EXPORT_FLAC: u32 = 1 << 2;
    }
}

/// Options for processing normal maps through the converter.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ImageNormalMapProcessing {
    None,
    OpenGl,
    DirectX,
}

/// Options for processing model materials.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum ModelMaterialProcessing {
    Skip,
    InModelFolder,
    InMaterialFolder,
}

/// Control scheme for preview viewport.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum PreviewControlScheme {
    Maya,
    Blender,
}

/// Global application settings.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    version: u32,
    load_settings: LoadSettings,
    model_settings: ModelSettings,
    model_material_processing: ModelMaterialProcessing,
    anim_settings: AnimSettings,
    audio_settings: AudioSettings,
    image_file_type: ImageFileType,
    image_normal_map_processing: ImageNormalMapProcessing,
    output_directory: Option<String>,
    preview_controls: PreviewControlScheme,
    preview_overlay: bool,
    auto_scale: bool,
    far_clip: u32,
    preview_window: bool,
    custom_scale: Option<f32>,
    last_files: Option<Vec<String>>,
    volume: u32,
    anim_bake: bool,
}

impl Settings {
    /// Loads the settings from the disk at the given path, or returns new ones.
    pub fn load(name: &str) -> Settings {
        std::fs::read_to_string(
            system::config_dir()
                .join(name.to_lowercase())
                .with_extension("dat"),
        )
        .map_or(Default::default(), |buffer| {
            miniserde::json::from_str(&buffer).unwrap_or_default()
        })
    }

    /// Saves the settings to the disk at the given path.
    pub fn save(&self, name: &str) {
        let result = miniserde::json::to_string(&self);

        let dirs = std::fs::create_dir_all(system::config_dir());

        debug_assert!(dirs.is_ok());

        let result = std::fs::write(
            system::config_dir()
                .join(name.to_lowercase())
                .with_extension("dat"),
            result,
        );

        debug_assert!(result.is_ok());
    }

    /// Checks whether or not the new settings requires a reload.
    pub fn reload_required(&self, new_settings: &Self) -> bool {
        if self.load_models() != new_settings.load_models()
            || self.load_animations() != new_settings.load_animations()
            || self.load_images() != new_settings.load_images()
            || self.load_materials() != new_settings.load_materials()
            || self.load_sounds() != new_settings.load_sounds()
            || self.load_raw_files() != new_settings.load_raw_files()
            || self.force_raw_files() != new_settings.force_raw_files()
        {
            return true;
        }

        false
    }

    /// Whether or not to load models.
    pub fn load_models(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_MODELS)
    }

    /// Sets whether or not to load models.
    pub fn set_load_models(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_MODELS, value);
    }

    /// Whether or not to load images.
    pub fn load_images(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_IMAGES)
    }

    /// Sets whether or not to load images.
    pub fn set_load_images(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_IMAGES, value)
    }

    /// Whether or not to load materials.
    pub fn load_materials(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_MATERIALS)
    }

    /// Sets whether or not to load materials.
    pub fn set_load_materials(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_MATERIALS, value)
    }

    /// Whether or not to load animations.
    pub fn load_animations(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_ANIMATIONS)
    }

    /// Sets whether or not to load animations.
    pub fn set_load_animations(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_ANIMATIONS, value)
    }

    /// Whether or not to load sounds.
    pub fn load_sounds(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_SOUNDS)
    }

    /// Sets whether or not to load sounds.
    pub fn set_load_sounds(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_SOUNDS, value)
    }

    /// Whether or not to load raw files.
    pub fn load_raw_files(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_RAW_FILES)
            || (cfg!(feature = "raw-files-forcible") && self.force_raw_files())
    }

    /// Sets whether or not to load raw files.
    pub fn set_load_raw_files(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_RAW_FILES, value)
    }

    /// Whether or not to force all assets as raw files.
    pub fn force_raw_files(&self) -> bool {
        self.load_settings
            .contains(LoadSettings::LOAD_FORCE_RAW_FILES)
    }

    /// Sets whether or not to force all assets as raw files.
    pub fn set_force_raw_files(&mut self, value: bool) {
        self.load_settings
            .set(LoadSettings::LOAD_FORCE_RAW_FILES, value)
    }

    /// The model file types to export to.
    pub fn model_file_types(&self) -> Vec<ModelFileType> {
        let mut result = Vec::with_capacity(8);

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_OBJ)
        {
            result.push(ModelFileType::Obj);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_SMD)
        {
            result.push(ModelFileType::Smd);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_XNA_LARA)
        {
            result.push(ModelFileType::XnaLara);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_XMODEL_EXPORT)
        {
            result.push(ModelFileType::XModelExport);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_CAST)
        {
            result.push(ModelFileType::Cast);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_MAYA)
        {
            result.push(ModelFileType::Maya);
        }

        if self
            .model_settings
            .contains(ModelSettings::EXPORT_FBX)
        {
            result.push(ModelFileType::Fbx);
        }

        result
    }

    /// Sets whether or not a model file type is in use.
    pub fn set_model_file_type(&mut self, file_type: ModelFileType, value: bool) {
        let flag = match file_type {
            ModelFileType::Obj => ModelSettings::EXPORT_OBJ,
            ModelFileType::Smd => ModelSettings::EXPORT_SMD,
            ModelFileType::XnaLara => ModelSettings::EXPORT_XNA_LARA,
            ModelFileType::XModelExport => ModelSettings::EXPORT_XMODEL_EXPORT,
            ModelFileType::Cast => ModelSettings::EXPORT_CAST,
            ModelFileType::Maya => ModelSettings::EXPORT_MAYA,
            ModelFileType::Fbx => ModelSettings::EXPORT_FBX,
        };

        self.model_settings.set(flag, value);
    }

    /// The model material processing technique.
    pub fn model_material_processing(&self) -> ModelMaterialProcessing {
        self.model_material_processing
    }

    /// Sets the model material processing.
    pub fn set_model_material_processing(&mut self, processing: ModelMaterialProcessing) {
        self.model_material_processing = processing;
    }

    /// The animation file types to export to.
    pub fn anim_file_types(&self) -> Vec<AnimationFileType> {
        let mut result = Vec::with_capacity(1);

        if self
            .anim_settings
            .contains(AnimSettings::EXPORT_CAST)
        {
            result.push(AnimationFileType::Cast);
        }

        result
    }

    /// Sets whether or not an anim file type is in use.
    pub fn set_anim_file_type(&mut self, file_type: AnimationFileType, value: bool) {
        let flag = match file_type {
            AnimationFileType::Cast => AnimSettings::EXPORT_CAST,
        };

        self.anim_settings.set(flag, value);
    }

    /// The audio file types to export to.
    pub fn audio_file_types(&self) -> Vec<AudioFileType> {
        let mut result = Vec::with_capacity(3);

        if self
            .audio_settings
            .contains(AudioSettings::EXPORT_WAV)
        {
            result.push(AudioFileType::Wav);
        }

        if self
            .audio_settings
            .contains(AudioSettings::EXPORT_FLAC)
        {
            result.push(AudioFileType::Flac);
        }

        result
    }

    /// Sets whether or not an audio file type is in use.
    pub fn set_audio_file_type(&mut self, file_type: AudioFileType, value: bool) {
        let flag = match file_type {
            AudioFileType::Wav => AudioSettings::EXPORT_WAV,
            AudioFileType::Flac => AudioSettings::EXPORT_FLAC,
            AudioFileType::Ogg => {
                // We don't support writing these formats.
                return;
            }
        };

        self.audio_settings.set(flag, value);
    }

    /// The image file type to export to.
    pub fn image_file_type(&self) -> ImageFileType {
        self.image_file_type
    }

    /// Sets the image file type to export to.
    pub fn set_image_file_type(&mut self, file_type: ImageFileType) {
        self.image_file_type = file_type;
    }

    /// The image normal map processing technique.
    pub fn image_normal_map_processing(&self) -> ImageNormalMapProcessing {
        self.image_normal_map_processing
    }

    /// Sets the image normal map processing.
    pub fn set_image_normal_map_processing(&mut self, processing: ImageNormalMapProcessing) {
        self.image_normal_map_processing = processing;
    }

    /// An output directory used to save assets.
    pub fn output_directory(&self) -> &Path {
        if let Some(output_directory) = &self.output_directory {
            return Path::new(output_directory);
        }

        system::output_dir()
    }

    /// Sets a new output directory.
    pub fn set_output_directory(&mut self, path: PathBuf) {
        self.output_directory = Some(path.to_string_lossy().into_owned());
    }

    /// Gets the preview control scheme.
    pub fn preview_controls(&self) -> PreviewControlScheme {
        self.preview_controls
    }

    /// Sets the preview control scheme.
    pub fn set_preview_controls(&mut self, controls: PreviewControlScheme) {
        self.preview_controls = controls;
    }

    /// Whether or not to show the preview overlay hints.
    pub fn preview_overlay(&self) -> bool {
        self.preview_overlay
    }

    /// Sets whether or not to show the preview overlay.
    pub fn set_preview_overlay(&mut self, value: bool) {
        self.preview_overlay = value;
    }

    /// Whether or not to automatically scale models and animations.
    pub fn auto_scale(&self) -> bool {
        self.auto_scale
    }

    /// Sets whether or not to automatically scale models and animations.
    pub fn set_auto_scale(&mut self, value: bool) {
        self.auto_scale = value;
    }

    /// Gets the far clip distance for preview.
    pub fn far_clip(&self) -> u32 {
        self.far_clip.clamp(10000, 1000000)
    }

    /// Sets the far clip distance for preview.
    pub fn set_far_clip(&mut self, far_clip: u32) {
        self.far_clip = far_clip;
    }

    /// Whether or not preview should open in a new window by default.
    pub fn preview_window(&self) -> bool {
        self.preview_window
    }

    /// Sets whether or not preview should open in a new window by default.
    pub fn set_preview_window(&mut self, value: bool) {
        self.preview_window = value;
    }

    /// Gets the custom auto scale value to use.
    pub fn custom_scale(&self) -> Option<f32> {
        self.custom_scale
    }

    /// Sets the custom auto scale value to use.
    pub fn set_custom_scale(&mut self, scale: Option<f32>) {
        self.custom_scale = scale;
    }

    /// Gets the automatic scale factor to use for assets, with the provided default scale.
    pub fn auto_scale_factor(&self, default: f32) -> Option<f32> {
        self.auto_scale()
            .then(|| self.custom_scale().unwrap_or(default))
    }

    /// Gets the last successfully loaded files.
    pub fn last_files(&self) -> Option<Vec<PathBuf>> {
        self.last_files
            .as_ref()
            .map(|last_files| {
                last_files
                    .iter()
                    .map(PathBuf::from)
                    .collect()
            })
    }

    /// Whether or not we have last files stored.
    pub fn has_last_files(&self) -> bool {
        self.last_files.is_some()
    }

    /// Sets the last successfully loaded files.
    pub fn set_last_files(&mut self, files: Option<Vec<PathBuf>>) {
        self.last_files = files.map(|files| {
            files
                .into_iter()
                .map(|file| file.to_string_lossy().into_owned())
                .collect()
        });
    }

    /// Gets the volume level for audio preview.
    pub fn volume(&self) -> u32 {
        self.volume
    }

    /// Sets the volume level for audio preview.
    pub fn set_volume(&mut self, volume: u32) {
        self.volume = volume.clamp(0, 50);
    }

    /// Gets whether or not to bake animations.
    pub fn anim_bake(&self) -> bool {
        self.anim_bake
    }

    /// Sets whether or not to bake animations.
    pub fn set_anim_bake(&mut self, value: bool) {
        self.anim_bake = value;
    }

    /// Update settings and returns a copy.
    pub fn update<F: FnOnce(&mut Self)>(&self, callback: F) -> Self {
        let mut settings = self.clone();

        callback(&mut settings);

        settings
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            load_settings: LoadSettings::from(
                LoadSettings::LOAD_MODELS
                    | LoadSettings::LOAD_IMAGES
                    | LoadSettings::LOAD_MATERIALS
                    | LoadSettings::LOAD_ANIMATIONS
                    | LoadSettings::LOAD_SOUNDS,
            ),
            model_settings: ModelSettings::from(ModelSettings::EXPORT_CAST),
            model_material_processing: ModelMaterialProcessing::InModelFolder,
            anim_settings: AnimSettings::from(AnimSettings::EXPORT_CAST),
            audio_settings: AudioSettings::from(AudioSettings::EXPORT_WAV),
            image_file_type: ImageFileType::Png,
            image_normal_map_processing: ImageNormalMapProcessing::OpenGl,
            output_directory: None,
            preview_controls: PreviewControlScheme::Maya,
            preview_overlay: true,
            auto_scale: true,
            far_clip: 10000,
            preview_window: false,
            custom_scale: None,
            last_files: None,
            volume: 30,
            anim_bake: false,
        }
    }
}
