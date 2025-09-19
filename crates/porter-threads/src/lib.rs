#![deny(unsafe_code)]

use std::sync::Once;
use std::thread::JoinHandle;

use rayon::Scope;

pub use rayon::iter::IndexedParallelIterator;
pub use rayon::iter::IntoParallelIterator;
pub use rayon::iter::ParallelIterator;
pub use rayon::slice::ParallelSlice;
pub use rayon::slice::ParallelSliceMut;
pub use rayon::str::ParallelString;

/// Used to run an error callback when a thread panics.
struct OnError<E>
where
    E: FnOnce() + Send + 'static,
{
    on_error: Option<E>,
}

impl<E> OnError<E>
where
    E: FnOnce() + Send + 'static,
{
    /// Constructs a new on error instance.
    pub fn new(on_error: E) -> Self {
        Self {
            on_error: Some(on_error),
        }
    }

    /// Clears the error instance.
    pub fn clear(&mut self) {
        self.on_error = None;
    }
}

impl<E> Drop for OnError<E>
where
    E: FnOnce() + Send + 'static,
{
    fn drop(&mut self) {
        if let Some(on_error) = self.on_error.take() {
            on_error();
        }
    }
}

/// Spawns the closure on a dedicated thread, with an error handler.
pub fn spawn_thread_with_error<F, T, E>(func: F, on_error: E) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
    E: FnOnce(),
    E: Send + 'static,
{
    std::thread::spawn(move || {
        let mut error = OnError::new(on_error);

        let result = func();

        error.clear();

        result
    })
}

/// Spawns the closure on a dedicated thread, with an error handler.
pub fn spawn_thread<F, T>(func: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(func)
}

/// Spawns the closure on the thread pool.
pub fn spawn<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    rayon::spawn(func)
}

/// Runs two closures in parellel and returns a pair of results.
pub fn join<A, B, RA, RB>(func_a: A, func_b: B) -> (RA, RB)
where
    A: FnOnce() -> RA + Send,
    B: FnOnce() -> RB + Send,
    RA: Send,
    RB: Send,
{
    rayon::join(func_a, func_b)
}

/// Calls a closure that can spawn sub tasks that are allowed to reference variables outside of the closure.
pub fn scope<'scope, OP, R>(op: OP) -> R
where
    OP: FnOnce(&Scope<'scope>) -> R + Send,
    R: Send,
{
    rayon::scope(op)
}

/// Ensures the thread pool has been initialized.
pub fn initialize_thread_pool() {
    static INITIALIZE: Once = Once::new();

    INITIALIZE.call_once(|| {
        let result = rayon::ThreadPoolBuilder::new()
            .num_threads(
                std::thread::available_parallelism()
                    .map(|threads| threads.get())
                    .unwrap_or_default()
                    .max(4),
            )
            .thread_name(|index| format!("porter-thread[{}]", index))
            .build_global();

        debug_assert!(result.is_ok());
    })
}
