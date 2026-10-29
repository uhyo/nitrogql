//! Stdio connection to TypeScript.
//!
//! The native build reads stdin and writes stdout itself.
//! The wasm build relies on the JavaScript host, which owns stdio as the wasm
//! module cannot wait for input without blocking the host.
//! Both builds frame messages here.

use super::framing::{MessageReader, header};

#[cfg(target_os = "wasi")]
#[link(wasm_import_module = "nitrogql_helper/content_mapper")]
unsafe extern "C" {
    /// Requests the next chunk of input. Result is asynchronously provided to given ticket.
    /// Ticket fails when the input is closed.
    fn read_input(ticket_handle: u32);
    /// Writes given bytes to the output.
    fn write_output(data_ptr: *const u8, data_len: usize);
}

/// Connection to TypeScript.
#[derive(Default)]
pub struct Connection {
    reader: MessageReader,
}

impl Connection {
    /// Receives the next JSON-RPC message.
    /// Returns None if the connection is closed.
    pub async fn receive(&mut self) -> Option<String> {
        loop {
            if let Some(message) = self.reader.next_message() {
                return Some(message);
            }
            let chunk = read_chunk().await?;
            self.reader.push(&chunk);
        }
    }

    /// Sends a JSON-RPC message.
    pub fn send(&mut self, message: &str) {
        write(&[header(message).as_bytes(), message.as_bytes()]);
    }
}

/// Reads the next chunk of input.
/// Returns None if the input is closed.
#[cfg(target_os = "wasi")]
async fn read_chunk() -> Option<Vec<u8>> {
    let ticket = nitrogql_async_runtime::issue_ticket();
    unsafe { read_input(ticket.id.into()) };
    ticket.await.ok()
}

/// Reads the next chunk of input.
/// Returns None if the input is closed.
#[cfg(not(target_os = "wasi"))]
async fn read_chunk() -> Option<Vec<u8>> {
    use std::io::{ErrorKind, Read, stdin};

    // Blocking is fine, as there is no other work to do while waiting for a message.
    let mut buffer = vec![0; 64 * 1024];
    loop {
        match stdin().read(&mut buffer) {
            Ok(0) => return None,
            Ok(len) => {
                buffer.truncate(len);
                return Some(buffer);
            }
            Err(err) if err.kind() == ErrorKind::Interrupted => continue,
            Err(err) => {
                log::error!("Failed to read stdin: {err}");
                return None;
            }
        }
    }
}

/// Writes given parts to the output in order.
#[cfg(target_os = "wasi")]
fn write(parts: &[&[u8]]) {
    for part in parts {
        unsafe { write_output(part.as_ptr(), part.len()) };
    }
}

/// Writes given parts to the output in order.
#[cfg(not(target_os = "wasi"))]
fn write(parts: &[&[u8]]) {
    use std::io::{Write, stdout};

    let mut stdout = stdout().lock();
    let result = parts
        .iter()
        .try_for_each(|part| stdout.write_all(part))
        .and_then(|()| stdout.flush());
    if let Err(err) = result {
        log::error!("Failed to write stdout: {err}");
    }
}
