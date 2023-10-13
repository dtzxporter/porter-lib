#[macro_use]
mod macros;
mod color;

pub use color::*;
pub use macros::*;

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

    STANDARD_STREAM.get_or_init(|| BufferWriter::stdout(ColorChoice::Always))
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

        if rec.EventType != KEY_EVENT as u16 && unsafe { rec.Event.KeyEvent.bKeyDown } > 0 {
            continue;
        } else {
            break;
        }
    }
}

/// Initializes the console, theme, and buffer sizes.
#[cfg(not(target_os = "windows"))]
pub fn initialize_console<T: AsRef<str>, D: AsRef<str>>(title: T, desc: D) {
    console!(header = "Info", <hr>);
    console!(header = "Info", "{}", title.as_ref());
    console!(header = "Info", { color = Color::White, "Author: "} , { color = Color::Pink, "DTZxPorter" }, { color = Color::White, " (https://dtzxporter.com)" });
    console!(header = "Info", "Desc: {}", desc.as_ref());
    console!(header = "Info", <hr>);
}

/// Initializes the console, theme, and buffer sizes.
#[cfg(target_os = "windows")]
pub fn initialize_console<T: AsRef<str>, D: AsRef<str>>(title: T, desc: D) {
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
    use widestring::U16CString;
    use windows_sys::Win32::System::Console::*;

    let Ok(str) = U16CString::from_str(title) else {
        panic!("invalid title");
    };

    unsafe { SetConsoleTitleW(str.as_ptr()) };

    let _buffer = standard_stream();

    let stdout = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };

    let mut screen_buffer: CONSOLE_SCREEN_BUFFER_INFOEX = unsafe { std::mem::zeroed() };
    screen_buffer.cbSize = std::mem::size_of_val(&screen_buffer) as u32;

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
