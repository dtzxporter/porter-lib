use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

struct AtomicSemaphoreInner {
    count: AtomicUsize,
    total: usize,
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
        let threads = std::thread::available_parallelism()
            .map(|threads| threads.get() / 2)
            .unwrap_or_default()
            .max(4);

        Self::with_max(threads)
    }

    /// Constructs a new semaphore.
    pub fn with_max(max: usize) -> Self {
        Self {
            inner: Arc::new(AtomicSemaphoreInner {
                count: AtomicUsize::new(max),
                total: max,
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
                        };
                    }
                    Err(count) => count,
                }
            }
        }
    }

    /// Checks if the semaphore is idle, and no locks are given out.
    pub fn is_idle(&self) -> bool {
        self.inner.count.load(Ordering::Relaxed) == self.inner.total
    }
}

impl Drop for AtomicSemaphoreGuard {
    fn drop(&mut self) {
        self.inner
            .count
            .fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for AtomicSemaphore {
    fn default() -> Self {
        Self::new()
    }
}
