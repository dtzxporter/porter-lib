use std::sync::Arc;

use iced::multi_window::Application;

use iced::Color;
use iced::Font;
use iced::Pixels;
use iced::Settings;

use crate::porter_main_settings;
use crate::PorterAssetManager;
use crate::PorterMain;
use crate::PorterMainColumn;

/// Used to build and configure the main window.
pub struct PorterMainBuilder {
    pub(crate) name: &'static str,
    pub(crate) version: &'static str,
    pub(crate) description: &'static str,
    pub(crate) file_filters: Vec<(String, Vec<String>)>,
    pub(crate) multi_file: bool,
    pub(crate) preview: bool,
    pub(crate) animations_enabled: bool,
    pub(crate) materials_enabled: bool,
    pub(crate) sounds_enabled: bool,
    pub(crate) sounds_convertable: bool,
    pub(crate) raw_files_enabled: bool,
    pub(crate) raw_files_forcable: bool,
    pub(crate) normal_map_converter: bool,
    pub(crate) columns: Vec<PorterMainColumn>,
    pub(crate) asset_manager: Arc<dyn PorterAssetManager>,
}

impl PorterMainBuilder {
    /// The name of the application. Used for the main window header, and the name of settings and crash files.
    pub fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// The version of the program.
    pub fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }

    /// The description of the program.
    pub fn description(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    /// Adds a column to the main asset view.
    pub fn column<H: Into<String>>(
        mut self,
        header: H,
        width: usize,
        color: Option<Color>,
    ) -> Self {
        self.columns
            .push(PorterMainColumn::new(header, width, color));
        self
    }

    /// Adds a file filter to the load files dialog.
    pub fn file_filter<T: Into<String>>(mut self, title: T, extensions: Vec<&'static str>) -> Self {
        self.file_filters.push((
            title.into(),
            extensions.into_iter().map(String::from).collect(),
        ));
        self
    }

    /// Enable or disable support for loading multiple files at once (Default: false).
    pub const fn multi_file(mut self, multi_file: bool) -> Self {
        self.multi_file = multi_file;
        self
    }

    /// Enable or disable the asset previewer (Default: true).
    pub const fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }

    /// Enable or disable animation support (Default: false).
    pub const fn animations_enabled(mut self, animations: bool) -> Self {
        self.animations_enabled = animations;
        self
    }

    /// Enable or disable material support (Default: false).
    pub const fn materials_enabled(mut self, materials: bool) -> Self {
        self.materials_enabled = materials;
        self
    }

    /// Enable or disable sounds support (Default: false).
    pub const fn sounds_enabled(mut self, sounds: bool) -> Self {
        self.sounds_enabled = sounds;
        self
    }

    /// Enable or disable sound conversion support (Default: true).
    pub const fn sounds_convertable(mut self, convertable: bool) -> Self {
        self.sounds_convertable = convertable;
        self
    }

    /// Enables or disables raw file support (Default: false).
    pub const fn raw_files_enabled(mut self, raw_files: bool) -> Self {
        self.raw_files_enabled = raw_files;
        self
    }

    /// Enables or disables support for forcing raw files (Default: false).
    pub const fn raw_files_forcable(mut self, forcable: bool) -> Self {
        self.raw_files_forcable = forcable;
        self
    }

    /// Enables or disables the normal map converter support (Default: true).
    pub const fn normal_map_converter(mut self, converter: bool) -> Self {
        self.normal_map_converter = converter;
        self
    }

    /// Runs the main window until it closes.
    pub fn run(self) {
        let settings = Settings {
            id: None,
            window: porter_main_settings(),
            flags: self,
            fonts: Vec::new(),
            default_font: Font::DEFAULT,
            default_text_size: if cfg!(target_os = "windows") {
                Pixels(14.0)
            } else {
                Pixels(16.0)
            },
            antialiasing: true,
        };

        PorterMain::run(settings).unwrap();
    }
}

/// Creates a new main window builder with the given asset manager.
pub fn create_main<A: PorterAssetManager + 'static>(asset_manager: A) -> PorterMainBuilder {
    PorterMainBuilder {
        name: "<unset>",
        version: "<unset>",
        description: "<unset>",
        file_filters: Vec::new(),
        multi_file: false,
        preview: true,
        animations_enabled: false,
        materials_enabled: false,
        sounds_enabled: false,
        sounds_convertable: true,
        raw_files_enabled: false,
        raw_files_forcable: false,
        normal_map_converter: true,
        columns: Vec::new(),
        asset_manager: Arc::new(asset_manager),
    }
}
