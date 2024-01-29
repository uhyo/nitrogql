use runtime::Runtime;

mod runtime;
mod ticket;

pub use ticket::{issue_string_ticket, Ticket};

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
