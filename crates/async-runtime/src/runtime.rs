use std::{
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    process,
    task::{Context, Poll},
};

use async_executor::LocalExecutor;

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
        self.inner.spawn(ExitOnPanic(Box::pin(future))).detach();
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

/// Exits the process when the inner future panics.
/// The executor catches panics of detached tasks, which would otherwise
/// end the process as if it had finished successfully.
struct ExitOnPanic<F>(Pin<Box<F>>);

impl<F: Future<Output = ()>> Future for ExitOnPanic<F> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        match catch_unwind(AssertUnwindSafe(|| self.0.as_mut().poll(cx))) {
            Ok(poll) => poll,
            // The panic message has already been printed by the panic hook.
            // 101 is the exit code of a panicking Rust program.
            Err(_) => process::exit(101),
        }
    }
}
