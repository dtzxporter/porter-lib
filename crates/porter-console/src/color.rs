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
        let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let mut mode: CONSOLE_MODE = unsafe { std::mem::zeroed() };

        unsafe { GetConsoleMode(stdout, &mut mode as *mut _) };

        mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING == ENABLE_VIRTUAL_TERMINAL_PROCESSING
    })
}

impl From<Color> for TColor {
    fn from(value: Color) -> Self {
        if color_mode() {
            match value {
                Color::Red => Self::Rgb(243, 68, 54),
                Color::Blue => Self::Rgb(0x27, 0x9B, 0xD4),
                Color::Green => Self::Rgb(0, 213, 133),
                Color::Orange => Self::Rgb(255, 152, 0),
                Color::Yellow => Self::Rgb(244, 246, 0),
                Color::Pink => Self::Rgb(255, 0, 208),
                Color::DarkGray => Self::Rgb(35, 35, 35),
                Color::White => Self::Rgb(255, 255, 255),
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
