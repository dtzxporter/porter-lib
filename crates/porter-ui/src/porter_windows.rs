use iced::window::settings::PlatformSpecific;

use iced::window::Position;
use iced::window::Settings;

use iced::Size;

/// Utility to create the main window settings.
pub fn porter_main_settings() -> Settings {
    Settings {
        size: Size::new(920.0, 582.0),
        position: Position::Centered,
        min_size: Some(Size::new(920.0, 582.0)),
        visible: false,
        ..Default::default()
    }
}

/// Utility to create the splash window settings.
pub fn porter_splash_settings() -> Settings {
    Settings {
        size: Size::new(865.0, 570.0),
        position: Position::Centered,
        min_size: Some(Size::new(865.0, 570.0)),
        decorations: false,
        resizable: false,
        platform_specific: PlatformSpecific {
            #[cfg(target_os = "windows")]
            skip_taskbar: true,
            #[cfg(target_os = "windows")]
            drag_and_drop: false,
            ..Default::default()
        },
        ..Default::default()
    }
}
