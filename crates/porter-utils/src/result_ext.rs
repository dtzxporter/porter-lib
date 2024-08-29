/// Utilities for logging errors on result types.
pub trait ResultExt<T, E> {
    /// Logs a debug message when an error occurs only for debug builds.
    fn debug(self, message: String) -> Result<T, E>;
    /// Logs a debug message returned from a callback when an error occurs only in debug builds.
    fn debug_fn<F: FnOnce(&E) -> String>(self, cb: F) -> Result<T, E>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    #[inline(always)]
    fn debug(self, message: String) -> Result<T, E> {
        #[cfg(debug_assertions)]
        {
            if self.is_err() {
                println!("[Debug] {}", message);
            }
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = message;
        }

        self
    }

    #[inline(always)]
    fn debug_fn<F: FnOnce(&E) -> String>(self, cb: F) -> Result<T, E> {
        #[cfg(debug_assertions)]
        {
            if let Err(e) = &self {
                println!("[Debug] {}", cb(e));
            }
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = cb;
        }

        self
    }
}
