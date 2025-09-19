use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

/// A trait used to clean paths, file names, or strings.
pub trait SanitizeExt {
    type T;

    /// Sanitizes a path, file name, or string and replaces invalid characters with "_".
    fn sanitized(&self) -> Self::T;
}

impl SanitizeExt for Path {
    type T = PathBuf;

    fn sanitized(&self) -> Self::T {
        let mut path = PathBuf::with_capacity(self.as_os_str().len());

        for component in self.components() {
            if let Component::Normal(part) = component {
                path.push(sanitize_str(&part.to_string_lossy()));
            } else {
                path.push(component);
            }
        }

        path
    }
}

impl SanitizeExt for PathBuf {
    type T = PathBuf;

    fn sanitized(&self) -> Self::T {
        self.as_path().sanitized()
    }
}

impl SanitizeExt for String {
    type T = String;

    fn sanitized(&self) -> Self::T {
        sanitize_str(self)
    }
}

/// Internal method to sanitize a string slice.
fn sanitize_str(string: &str) -> String {
    let mut global: String = string
        .chars()
        .map(|ch| match ch {
            // Illegal characters.
            '/' | '?' | '<' | '>' | '\\' | ':' | '*' | '|' | '"' => '_',
            // Control characters.
            '\u{0000}'..='\u{001F}' | '\u{0080}'..='\u{009F}' => '_',
            // Safe character.
            _ => ch,
        })
        // Leading '.' characters.
        .scan(true, |leading, char| {
            if *leading && char == '.' {
                Some('_')
            } else {
                *leading = false;

                Some(char)
            }
        })
        .collect();

    if cfg!(target_os = "windows") {
        // Windows specific trailing whitespace/period characters.
        let mut popped = 0;

        while global
            .chars()
            .last()
            .is_some_and(|ch| ch.is_whitespace() || ch == '.')
        {
            global.pop();
            popped += 1;
        }

        for _ in 0..popped {
            global.push('_');
        }

        // Windows specific reserved file names.
        if global.eq_ignore_ascii_case("con")
            || global.eq_ignore_ascii_case("prn")
            || global.eq_ignore_ascii_case("aux")
            || global.eq_ignore_ascii_case("nul")
        {
            format!("_{global}")
        } else {
            global
        }
    } else {
        global
    }
}
