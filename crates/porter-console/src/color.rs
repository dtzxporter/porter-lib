use termcolor::Color as TColor;

/// One of the built in console colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Blue,
    Green,
    Orange,
    Yellow,
    Pink,
    DarkGray,
    White,
}

/// Whether or not the console supports terminal sequences.
#[cfg(not(target_os = "windows"))]
fn color_mode() -> bool {
    true
}

/// Whether or not the console supports terminal sequences.
#[cfg(target_os = "windows")]
fn color_mode() -> bool {
    use std::sync::OnceLock;
    use windows_sys::Win32::System::Console::*;

    static MODE: OnceLock<bool> = OnceLock::new();

    *MODE.get_or_init(|| {
        // SAFETY: It's safe to pass INVALID_HANDLE_VALUE to `GetConsoleMode` which will return an error.
        let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        // SAFETY: Initializing CONSOLE_MODE to 0 is default value.
        let mut mode: CONSOLE_MODE = unsafe { std::mem::zeroed() };

        // SAFETY: If this method fails, mode was initialized to 0, and will have the default configuration.
        unsafe { GetConsoleMode(stdout, &mut mode as *mut _) };

        mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == ENABLE_VIRTUAL_TERMINAL_PROCESSING
    })
}

impl From<Color> for TColor {
    fn from(value: Color) -> Self {
        if color_mode() {
            match value {
                Color::Red => Self::Rgb(0xF3, 0x44, 0x36),
                Color::Blue => Self::Rgb(0x27, 0x9B, 0xD4),
                Color::Green => Self::Rgb(0x00, 0xD5, 0x85),
                Color::Orange => Self::Rgb(0xFF, 0x98, 0x00),
                Color::Yellow => Self::Rgb(0xF4, 0xF6, 0x00),
                Color::Pink => Self::Rgb(0xFF, 0x00, 0xD0),
                Color::DarkGray => Self::Rgb(0x23, 0x23, 0x23),
                Color::White => Self::Rgb(0xFF, 0xFF, 0xFF),
            }
        } else {
            match value {
                Color::Red => Self::Ansi256(0),
                Color::Blue => Self::Ansi256(4),
                Color::Green => Self::Ansi256(2),
                Color::Orange => Self::Ansi256(1),
                Color::Yellow => Self::Ansi256(6),
                Color::Pink => Self::Ansi256(5),
                Color::DarkGray => Self::Ansi256(3),
                Color::White => Self::Ansi256(7),
            }
        }
    }
}

#[cfg(target_os = "windows")]
macro_rules! windows_rgb {
    ($r:expr, $g:expr, $b:expr) => {
        (($r as u32) << 16) | (($g as u32) << 8) | ($b as u32)
    };
}

#[cfg(target_os = "windows")]
impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        let TColor::Rgb(r, g, b) = TColor::from(value) else {
            return 0;
        };

        windows_rgb!(r, g, b)
    }
}
