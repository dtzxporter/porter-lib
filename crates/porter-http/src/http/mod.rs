#[cfg(any(target_os = "macos", target_os = "linux"))]
mod curl;
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub use curl::*;

#[cfg(target_os = "windows")]
mod winhttp;
#[cfg(target_os = "windows")]
pub use winhttp::*;
