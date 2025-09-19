#![deny(unsafe_code)]

mod utilities;

pub(crate) use utilities::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Used to configure windows resources on the resulting binary.
#[allow(unused)]
pub fn configure_windows<I: Into<String>>(icon: I, admin: bool) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    windows::configure(icon, admin)?;
    Ok(())
}

/// Used to configure macOS resources on the resulting binary.
#[allow(unused)]
pub fn configure_macos<I: Into<String>>(icon: I) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    macos::configure(icon)?;
    Ok(())
}

/// Used to configure linux resources on the resulting binary.
#[allow(unused)]
pub fn configure_linux<I: Into<String>>(icon: I) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    linux::configure(icon)?;
    Ok(())
}
