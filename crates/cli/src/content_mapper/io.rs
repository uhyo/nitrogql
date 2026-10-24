//! Bridge to the JavaScript host, which owns the stdio connection to TypeScript.

use nitrogql_async_runtime::issue_string_ticket;

#[cfg(target_os = "wasi")]
#[link(wasm_import_module = "nitrogql_helper/content_mapper")]
unsafe extern "C" {
    /// Requests the next message from TypeScript. Result is asynchronously provided to given ticket.
    /// Ticket fails when the connection is closed.
    fn read_message(ticket_handle: u32);
    /// Sends a message to TypeScript.
    fn write_message(message_ptr: *const u8, message_len: usize);
}

#[cfg(not(target_os = "wasi"))]
unsafe fn read_message(_ticket_handle: u32) {
    panic!("Not implemented")
}

#[cfg(not(target_os = "wasi"))]
unsafe fn write_message(_message_ptr: *const u8, _message_len: usize) {
    panic!("Not implemented")
}

/// Receives the next JSON-RPC message.
/// Returns None if the connection is closed.
pub async fn receive() -> Option<String> {
    let ticket = issue_string_ticket();
    unsafe { read_message(ticket.id.into()) };
    ticket.await.ok()
}

/// Sends a JSON-RPC message.
pub fn send(message: &str) {
    unsafe { write_message(message.as_ptr(), message.len()) };
}
