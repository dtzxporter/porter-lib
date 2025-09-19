#[macro_use]
mod macros;
mod color;

pub use color::*;

pub use pico_args::Arguments;
pub use pico_args::Error as PicoError;

use std::io::Write;
use std::sync::OnceLock;

use termcolor::BufferWriter;
use termcolor::ColorChoice;
use termcolor::ColorSpec;
use termcolor::WriteColor;

/// Gets the standard output stream.
pub(crate) fn standard_stream() -> &'static BufferWriter {
    static STANDARD_STREAM: OnceLock<BufferWriter> = OnceLock::new();

    STANDARD_STREAM.get_or_init(|| BufferWriter::stdout(ColorChoice::Auto))
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct _FormatOp<'a> {
    pub foreground: Color,
    pub background: Option<Color>,
    pub args: std::fmt::Arguments<'a>,
}

#[doc(hidden)]
pub fn _write_header(header: &'static str, format_ops: &[_FormatOp<'_>], new_line: bool) {
    let write = || -> Result<(), std::io::Error> {
        let stdout = standard_stream();
        let mut buffer = stdout.buffer();

        buffer.set_color(
            ColorSpec::new()
                .set_bg(Some(Color::DarkGray.into()))
                .set_fg(Some(Color::Blue.into())),
        )?;

        write!(
            &mut buffer,
            "{:width$}",
            format!("[{}]", header),
            width = 20
        )?;

        buffer.set_color(
            ColorSpec::new()
                .set_bg(None)
                .set_fg(Some(Color::White.into())),
        )?;

        write!(&mut buffer, ": ")?;

        for format_op in format_ops {
            buffer.set_color(
                ColorSpec::new()
                    .set_bg(format_op.background.map(Into::into))
                    .set_fg(Some(format_op.foreground.into())),
            )?;

            buffer.write_fmt(format_op.args)?;
        }

        buffer.set_color(
            ColorSpec::new()
                .set_bg(None)
                .set_fg(Some(Color::White.into())),
        )?;

        if new_line {
            writeln!(&mut buffer)?;
        }

        stdout.print(&buffer)?;

        Ok(())
    };

    if let Err(e) = write() {
        panic!("failed printing to stdout: {e}");
    }
}

/// Informs the user they must press enter to continue.
#[cfg(not(target_os = "windows"))]
pub fn press_any_key() {
    // This doesn't make sense on non-windows platforms.
}

#[cfg(target_os = "windows")]
mod win32 {
    use windows_sys::Win32::System::Console::*;

    /// Alt key code.
    pub const ALT_VK_CODE: u16 = 0x12;

    /// Utility to check if a key is down.
    pub fn is_keydown_event(record: &INPUT_RECORD) -> bool {
        // SAFETY: The caller must uphold that the event is a KeyEvent.
        record.EventType == KEY_EVENT as u16 && unsafe { record.Event.KeyEvent.bKeyDown } > 0
    }

    /// Utility to check if a mod key is down.
    pub fn is_mod_key(record: &INPUT_RECORD) -> bool {
        // SAFETY: The caller must uphold that the event is a KeyEvent.
        let key_code = unsafe { record.Event.KeyEvent.wVirtualKeyCode };

        (0x10..=0x12).contains(&key_code)
            || key_code == 0x14
            || key_code == 0x90
            || key_code == 0x91
    }
}

/// Informs the user they must press enter to continue.
#[cfg(target_os = "windows")]
pub fn press_any_key() {
    use windows_sys::Win32::System::Console::*;

    console!(press_any_key);

    let _ = std::io::stdout().flush();

    let stdin = unsafe { GetStdHandle(STD_INPUT_HANDLE) };

    unsafe { FlushConsoleInputBuffer(stdin) };

    let mut rec: INPUT_RECORD = INPUT_RECORD {
        EventType: 0,
        Event: unsafe { std::mem::zeroed() },
    };

    let mut read = 0;

    loop {
        unsafe { ReadConsoleInputW(stdin, &mut rec, 1, &mut read) };

        let key_code = unsafe { rec.Event.KeyEvent.wVirtualKeyCode };

        if !win32::is_keydown_event(&rec) && key_code != win32::ALT_VK_CODE {
            continue;
        }

        let ch = unsafe { rec.Event.KeyEvent.uChar.AsciiChar } as i8;

        if ch == 0 && win32::is_mod_key(&rec) {
            continue;
        }

        break;
    }
}

/// Initializes the console, theme, and buffer sizes.
pub fn initialize_console<T: AsRef<str>, D: AsRef<str>>(title: T, desc: D) {
    #[cfg(target_os = "windows")]
    setup_windows_console(title.as_ref());

    console!(header = "Info", <hr>);
    console!(header = "Info", "{}", title.as_ref());
    console!(header = "Info", { color = Color::White, "Author: "} , { color = Color::Pink, "DTZxPorter" }, { color = Color::White, " (https://dtzxporter.com)" });
    console!(header = "Info", "Desc: {}", desc.as_ref());
    console!(header = "Info", <hr>);
}

/// Utility to configure the windows console.
#[cfg(target_os = "windows")]
fn setup_windows_console(title: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::System::Console::*;

    let title: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0x0)).collect();

    unsafe { SetConsoleTitleW(title.as_ptr()) };

    let _buffer = standard_stream();

    let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };

    let mut screen_buffer: CONSOLE_SCREEN_BUFFER_INFOEX = unsafe { std::mem::zeroed() };
    screen_buffer.cbSize = size_of_val(&screen_buffer) as u32;

    unsafe { GetConsoleScreenBufferInfoEx(stdout, &mut screen_buffer as *mut _) };

    screen_buffer.srWindow.Bottom += 1;
    screen_buffer.srWindow.Right += 1;

    for i in 0..8 {
        screen_buffer.ColorTable[i] = match i {
            0 => Color::Red.into(),
            1 => Color::Blue.into(),
            2 => Color::Green.into(),
            3 => Color::Orange.into(),
            4 => Color::Yellow.into(),
            5 => Color::Pink.into(),
            6 => Color::DarkGray.into(),
            7 => Color::White.into(),
            _ => unreachable!(),
        };
    }

    unsafe { SetConsoleScreenBufferInfoEx(stdout, &screen_buffer as *const _) };
}
