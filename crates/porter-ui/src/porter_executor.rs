use iced::futures::executor::ThreadPool;
use iced::futures::Future;

/// An async future executor using a threadpool.
pub struct PorterExecutor {
    inner: ThreadPool,
}

impl iced::Executor for PorterExecutor {
    fn new() -> Result<Self, iced::futures::io::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            inner: ThreadPool::builder().pool_size(2).create()?,
        })
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.inner.spawn_ok(future)
    }
}
