use nitrogql_async_runtime::issue_string_ticket;
use thiserror::Error;

#[cfg(target_os = "wasi")]
#[link(wasm_import_module = "nitrogql_helper/config")]
extern "C" {
    /// Executes given code. Result is asynchronously provided to given ticket.
    fn execute_node(code_ptr: *const u8, code_len: usize, ticket_handle: u32);
}

#[cfg(not(target_os = "wasi"))]
unsafe fn execute_node(_code_ptr: *const u8, _code_len: usize, _ticket_handle: u32) {
    panic!("Not implemented")
}

#[derive(Debug, Error)]
pub enum ExecuteConfigError {
    #[error("Failed to execute config file")]
    FailedToExecuteConfigFile,
    #[error("Failed to read result")]
    FailedToReadResult,
}

/// Executes given code using Node.js and returns the result.
/// Result is a JSON string containing the value exported as default export.
pub async fn execute_js(code: &str) -> Result<String, ExecuteConfigError> {
    let ticket = issue_string_ticket();
    unsafe { execute_node(code.as_ptr(), code.len(), ticket.id.into()) };

    let result = ticket.await;
    let Ok(result) = result else {
        return Err(ExecuteConfigError::FailedToExecuteConfigFile);
    };

    Ok(result)
}
