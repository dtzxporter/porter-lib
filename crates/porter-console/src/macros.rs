/// Prints a line to the console with a colored header, and variable format instructions.
///
/// Each instruction can have a color, and background color specified.
#[macro_export]
macro_rules! console {
    (header = $vheader:expr, $({color = $vcolor:expr, background = $vbackground:expr, $($varg:tt)*}),*) => {{
        $crate::_write_header($vheader, &[
            $($crate::_FormatOp {
                foreground: $vcolor,
                background: $vbackground,
                args: ::std::format_args!($($varg)*,)
            }),*
        ], true);
    }};
    (header = $vheader:expr, $({color = $vcolor:expr, $($varg:tt)*}),*) => {{
        $crate::_write_header($vheader, &[
            $($crate::_FormatOp {
                foreground: $vcolor,
                background: None,
                args: ::std::format_args!($($varg)*,)
            }),*
        ], true);
    }};
    (header = $vheader:expr, <hr>) => {{
        $crate::_write_header($vheader, &[
            $crate::_FormatOp {
                foreground: $crate::Color::White,
                background: None,
                args: ::std::format_args!("-----------------------------------------------------------------------------------------")
            }
        ], true);
    }};
    (header = $vheader:expr, $($arg:tt)*) => {{
        $crate::_write_header($vheader, &[$crate::_FormatOp {
            foreground: $crate::Color::White,
            background: None,
            args: ::std::format_args!($($arg)*,)
        }], true);
    }};
    (press_any_key) => {{
        $crate::_write_header("User Input", &[$crate::_FormatOp {
            foreground: $crate::Color::White,
            background: None,
            args: ::std::format_args!("Press any key to continue...")
        }], false);
    }};
    ($($arg:tt)*) => {{
        $crate::console!(header = "Info", $($arg)*);
    }};
}
