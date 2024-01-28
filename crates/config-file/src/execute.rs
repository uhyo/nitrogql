use std::{
    cell::{Cell, RefCell},
    future::poll_fn,
    task::{Poll, Waker},
};

use thiserror::Error;

#[cfg(target_os = "wasi")]
#[link(wasm_import_module = "nitrogql_helper/config")]
extern "C" {
    /// Executes given code and returns the handle to result.
    fn execute_node(code_ptr: *const u8, code_len: usize) -> u32;
    /// Returns the length of result of executing config file.
    fn result_len(handle: u32) -> usize;
    /// Writes result of executing config file to given buffer.
    fn write_result_to_buffer(handle: u32, buffer: *mut u8, buffer_len: usize) -> usize;
    /// Frees memory allocated for result of executing config file.
    fn free_result(handle: u32);
}

#[cfg(not(target_os = "wasi"))]
unsafe fn execute_node(_code_ptr: *const u8, _code_len: usize) -> u32 {
    panic!("Not implemented")
}

#[cfg(not(target_os = "wasi"))]
unsafe fn result_len(_handle: u32) -> usize {
    panic!("Not implemented")
}

#[cfg(not(target_os = "wasi"))]
unsafe fn write_result_to_buffer(_handle: u32, _buffer: *mut u8, _buffer_len: usize) -> usize {
    panic!("Not implemented")
}

#[cfg(not(target_os = "wasi"))]
unsafe fn free_result(_handle: u32) {
    panic!("Not implemented")
}

#[derive(Debug, Error)]
pub enum ExecuteConfigError {
    #[error("Failed to execute config file")]
    FailedToExecuteConfigFile,
    #[error("Failed to read result")]
    FailedToReadResult,
}

thread_local! {
    static PENDING_HANDLE: Cell<u32> = Cell::new(0);
    /// Result provided by `execute_node_ret`.
    /// `None` means that the result is not yet available.
    /// `Some(true)` means that the result is available and it is Ok.
    /// `Some(false)` means that the result is available and it is Err.
    static RESULT: Cell<Option<bool>> = Cell::new(None);
    static WAKER: RefCell<Option<Waker>> = RefCell::new(None);
}

/// Executes given code using Node.js and returns the result.
pub async fn execute_js(code: &str) -> Result<String, ExecuteConfigError> {
    let handle = unsafe { execute_node(code.as_ptr(), code.len()) };
    if handle == 0 {
        return Err(ExecuteConfigError::FailedToExecuteConfigFile);
    }
    // wait for the callback (execute_node_ret) to be called
    PENDING_HANDLE.with(|pending_handle| {
        pending_handle.set(handle);
    });
    RESULT.with(|result| result.set(None));
    let future = poll_fn(|ctx| {
        RESULT.with(|result| {
            if let Some(r) = result.get() {
                return Poll::Ready(r);
            }
            let waker = ctx.waker();
            WAKER.with(|waker_cell| {
                *waker_cell.borrow_mut() = Some(waker.clone());
            });
            Poll::Pending
        })
    });

    let result = future.await;
    if !result {
        return Err(ExecuteConfigError::FailedToExecuteConfigFile);
    }

    let result_len = unsafe { result_len(handle) };
    let mut result = vec![0; result_len];
    let written_len = unsafe { write_result_to_buffer(handle, result.as_mut_ptr(), result_len) };
    if written_len != result_len {
        return Err(ExecuteConfigError::FailedToReadResult);
    }
    unsafe { free_result(handle) };
    Ok(String::from_utf8(result).unwrap())
}
