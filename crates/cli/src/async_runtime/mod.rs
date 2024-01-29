use async_executor::LocalExecutor;

thread_local! {
    static RUNTIME: Runtime = Runtime::new();
}

pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    RUNTIME.with(|runtime| runtime.spawn(future));
}

pub fn drive() {
    RUNTIME.with(|runtime| runtime.drive());
}

/// Simple async runtime used by the CLI.
pub struct Runtime {
    inner: LocalExecutor<'static>,
}

impl Runtime {
    /// Create a new async runtime.
    pub fn new() -> Self {
        Self {
            inner: LocalExecutor::new(),
        }
    }

    /// Spawn a future onto the runtime.
    pub fn spawn<F>(&self, future: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        self.inner.spawn(future).detach();
    }

    /// Drive the runtime until there is no more work to do.
    pub fn drive(&self) {
        loop {
            if !self.inner.try_tick() {
                break;
            }
        }
    }
}
