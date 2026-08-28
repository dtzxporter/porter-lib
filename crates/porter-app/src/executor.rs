use iced::futures::Future;
use iced::futures::executor::ThreadPool;
use iced::futures::io::Error;

/// An async future executor using a threadpool.
pub struct Executor {
    inner: ThreadPool,
}

impl iced::Executor for Executor {
    fn new() -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Self {
            inner: ThreadPool::builder()
                // Limit the thread count to 2 to prevent resource starvation.
                .pool_size(2)
                .create()?,
        })
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.inner.spawn_ok(future)
    }

    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        self.inner.block_on(future)
    }
}
