use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

struct AtomicProgressInner {
    total: AtomicUsize,
    complete: AtomicUsize,
}

/// Used to track progress across multi-threading operations.
#[repr(transparent)]
#[derive(Clone)]
pub struct AtomicProgress {
    inner: Arc<AtomicProgressInner>,
}

impl AtomicProgress {
    /// Constructs a new instance of atomic progress.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AtomicProgressInner {
                total: AtomicUsize::new(0),
                complete: AtomicUsize::new(0),
            }),
        }
    }

    /// Resets the progress to the new total.
    pub fn reset(&self, total: usize) {
        self.inner.total.store(total, Ordering::Relaxed);
        self.inner.complete.store(0, Ordering::Relaxed);
    }

    /// Increments the completed count.
    pub fn increment(&self) {
        self.inner.complete.fetch_add(1, Ordering::Relaxed);
    }

    /// Gets the progress value out of 100%.
    pub fn progress(&self) -> u32 {
        let completed = self.inner.complete.load(Ordering::Relaxed);
        let total = self.inner.total.load(Ordering::Relaxed);

        if completed == 0 {
            return 0;
        }

        (((completed as f32) / (total as f32) * 100.0) as u32)
            .min(100)
            .max(0)
    }
}

impl Default for AtomicProgress {
    fn default() -> Self {
        Self::new()
    }
}
