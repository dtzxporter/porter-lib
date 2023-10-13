use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

struct AtomicSemaphoreInner {
    count: AtomicUsize,
}

/// An atomic semaphore.
#[derive(Clone)]
pub struct AtomicSemaphore {
    inner: Arc<AtomicSemaphoreInner>,
}

/// Used to sync the semaphore.
pub struct AtomicSemaphoreGuard {
    inner: Arc<AtomicSemaphoreInner>,
}

impl AtomicSemaphore {
    /// Constructs a new semaphore with the default thread count.
    pub fn new() -> Self {
        Self::with_max(num_cpus::get_physical())
    }

    /// Constructs a new semaphore.
    pub fn with_max(max: usize) -> Self {
        Self {
            inner: Arc::new(AtomicSemaphoreInner {
                count: AtomicUsize::new(max),
            }),
        }
    }

    /// Waits for the next available lock.
    pub fn wait(&self) -> AtomicSemaphoreGuard {
        let mut count = self.inner.count.load(Ordering::Relaxed);

        loop {
            count = if count == 0 {
                std::thread::yield_now();

                self.inner.count.load(Ordering::Relaxed)
            } else {
                match self.inner.count.compare_exchange(
                    count,
                    count - 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return AtomicSemaphoreGuard {
                            inner: self.inner.clone(),
                        }
                    }
                    Err(count) => count,
                }
            }
        }
    }
}

impl Drop for AtomicSemaphoreGuard {
    fn drop(&mut self) {
        self.inner.count.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for AtomicSemaphore {
    fn default() -> Self {
        Self::new()
    }
}
