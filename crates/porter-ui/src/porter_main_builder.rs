use std::sync::Arc;

use iced::window::icon;
use iced::window::Icon;
use iced::window::Position;

use iced::Application;
use iced::Color;
use iced::Font;
use iced::Pixels;
use iced::Settings;
use iced::Size;

use crate::PorterAssetManager;
use crate::PorterMain;
use crate::PorterMainColumn;

/// Used to build and configure the main window.
pub struct PorterMainBuilder {
    pub(crate) title: String,
    pub(crate) name: String,
    pub(crate) donate_url: String,
    pub(crate) icon: Option<Icon>,
    pub(crate) file_filters: Vec<(String, Vec<String>)>,
    pub(crate) multi_file: bool,
    pub(crate) preview: bool,
    pub(crate) materials_enabled: bool,
    pub(crate) sounds_enabled: bool,
    pub(crate) raw_files_enabled: bool,
    pub(crate) raw_files_forcable: bool,
    pub(crate) columns: Vec<PorterMainColumn>,
    pub(crate) asset_manager: Arc<dyn PorterAssetManager>,
}

impl PorterMainBuilder {
    /// Sets the title of the main window.
    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    /// The name of the application. Used for the main window header, and the name of settings and crash files.
    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = name.into();
        self
    }

    /// The url to open when clicking the donation button.
    pub fn donate_url<T: Into<String>>(mut self, url: T) -> Self {
        self.donate_url = url.into();
        self
    }

    /// Loads the icon file from a resource.
    pub fn icon<T: AsRef<[u8]>>(mut self, icon: T) -> Self {
        self.icon = icon::from_file_data(icon.as_ref(), Some(image::ImageFormat::Ico)).ok();
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

    /// Enable or disable support for loading multiple files at once.
    pub const fn multi_file(mut self, multi_file: bool) -> Self {
        self.multi_file = multi_file;
        self
    }

    /// Enable or disable the asset previewer.
    pub const fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }

    /// Enable or disable material support.
    pub const fn materials_enabled(mut self, materials: bool) -> Self {
        self.materials_enabled = materials;
        self
    }

    /// Enable or disable sounds support.
    pub const fn sounds_enabled(mut self, sounds: bool) -> Self {
        self.sounds_enabled = sounds;
        self
    }

    /// Enables or disables raw file support.
    pub const fn raw_files_enabled(mut self, raw_files: bool) -> Self {
        self.raw_files_enabled = raw_files;
        self
    }

    /// Enables or disables support for forcing raw files.
    pub const fn raw_files_forcable(mut self, forcable: bool) -> Self {
        self.raw_files_forcable = forcable;
        self
    }

    /// Runs the main window until it closes.
    pub fn run(mut self) {
        let settings = Settings {
            id: None,
            window: iced::window::Settings {
                size: Size::new(920.0, 582.0),
                position: Position::Centered,
                min_size: Some(Size::new(920.0, 582.0)),
                icon: self.icon.take(),
                ..Default::default()
            },
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
        title: String::new(),
        name: String::new(),
        donate_url: String::new(),
        icon: None,
        file_filters: Vec::new(),
        multi_file: false,
        preview: true,
        materials_enabled: false,
        sounds_enabled: false,
        raw_files_enabled: false,
        raw_files_forcable: false,
        columns: Vec::new(),
        asset_manager: Arc::new(asset_manager),
    }
}
