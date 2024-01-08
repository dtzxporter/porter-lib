mod porter_asset_manager;
mod porter_asset_status;
mod porter_color_palette;
mod porter_divider;
mod porter_executor;
mod porter_main;
mod porter_main_about;
mod porter_main_builder;
mod porter_main_column;
mod porter_main_commands;
mod porter_main_events;
mod porter_main_settings;
mod porter_overlay;
mod porter_preview_asset;
mod porter_set_parent_windows;
mod porter_settings;
mod porter_text;
mod porter_theme;
mod porter_ui;
mod porter_viewport;

pub mod porter_easing;
pub mod porter_spinner;

pub use porter_asset_manager::*;
pub use porter_asset_status::*;
pub use porter_color_palette::*;
pub use porter_main_builder::*;
pub use porter_main_column::*;
pub use porter_preview_asset::*;
pub use porter_settings::*;
pub use porter_ui::*;

pub use iced::Color;

pub(crate) use porter_divider::*;
pub(crate) use porter_executor::*;
pub(crate) use porter_main::*;
pub(crate) use porter_overlay::*;
pub(crate) use porter_set_parent_windows::*;
pub(crate) use porter_text::*;
pub(crate) use porter_theme::*;
pub(crate) use porter_viewport::*;

use std::backtrace::Backtrace;

use directories::ProjectDirs;

/// Encrypts a string using the given key.
fn xor_encrypt<K: AsRef<[u8]>>(input: String, key: K) -> Vec<u8> {
    let key = key.as_ref();
    let mut buffer = input.as_bytes().to_vec();

    for i in 0..buffer.len() {
        buffer[i] ^= key[i % key.len()];
    }

    buffer
}

/// Installs a runtime panic hook, which will log an encrypted panic stack trace.
pub fn install_panic_hook<S: Into<String>>(name: S) {
    if cfg!(debug_assertions) {
        return;
    }

    if let Some(project_directory) = ProjectDirs::from("com", "DTZxPorter", "GameTools") {
        let target = project_directory
            .config_dir()
            .join(name.into().to_lowercase())
            .with_extension("crash");

        let _ = std::fs::create_dir_all(project_directory.config_dir());

        std::panic::set_hook(Box::new(move |error| {
            let backtrace = Backtrace::force_capture();
            let error = format!("{} {:?} ({})", error, backtrace, env!("CARGO_PKG_VERSION"));

            let _ = std::fs::write(target.clone(), xor_encrypt(error, "asakujaku"));
        }));
    }
}

/// Method to open a url in the user's default browser.
pub fn open_url<U: AsRef<str>>(url: U) {
    let url = url.as_ref();

    #[cfg(target_os = "windows")]
    {
        use widestring::U16CString;

        use windows_sys::Win32::UI::Shell::*;
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        let url = U16CString::from_str(url).expect("bad url");

        // SAFETY: The pointer to url lives as long as the call does, and is checked that it's a valid string,
        // in this case we do not care whether or not the call succeeds or fails.
        unsafe {
            ShellExecuteW(
                0,
                std::ptr::null(),
                url.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let result = Command::new("open").arg(url).output();

        debug_assert!(result.is_ok());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        use std::process::Command;

        let result = Command::new("xdg-open").arg(url).output();

        debug_assert!(result.is_ok());
    }
}
