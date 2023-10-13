use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Used to atomically cancel a multi-threaded operation.
#[repr(transparent)]
#[derive(Default, Clone)]
pub struct AtomicCancel {
    inner: Arc<AtomicBool>,
}

impl AtomicCancel {
    /// Constructs a new atomic cancel.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Resets the value of the canceller.
    pub fn reset(&self) {
        self.inner.store(false, Ordering::Relaxed);
    }

    /// Signals that the operation is cancelled.
    pub fn cancel(&self) {
        self.inner.store(true, Ordering::Relaxed);
    }

    /// Whether or not the operation is cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::Relaxed)
    }
}
