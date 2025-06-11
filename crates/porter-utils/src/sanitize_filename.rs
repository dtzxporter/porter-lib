use sanitize_filename::Options;
use sanitize_filename::sanitize_with_options;

/// A trait used to clean a file name.
pub trait SanitizeFilename {
    /// Sanitizes a file name and replaces invalid characters with "_".
    fn sanitized(&self) -> String;
}

impl SanitizeFilename for String {
    fn sanitized(&self) -> String {
        sanitize_with_options(
            self,
            Options {
                windows: true,
                truncate: false,
                replacement: "_",
            },
        )
    }
}
