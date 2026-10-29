use runtime::Runtime;

mod runtime;
// Tickets let the wasm build wait on results from the JavaScript host.
// The native build waits with blocking calls instead.
#[cfg(target_os = "wasi")]
mod ticket;

#[cfg(target_os = "wasi")]
pub use ticket::{Ticket, issue_ticket};

thread_local! {
    static RUNTIME: Runtime = Runtime::new();
}

/// Spawn a future onto the runtime.
pub fn spawn<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    RUNTIME.with(|runtime| runtime.spawn(future));
}

/// Drive the runtime until there is no more work to do.
pub fn drive() {
    RUNTIME.with(|runtime| runtime.drive());
}
