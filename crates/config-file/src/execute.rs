use std::path::Path;

use thiserror::Error;

#[cfg(target_os = "wasi")]
#[link(wasm_import_module = "nitrogql_helper/config")]
extern "C" {
    /// Executes given config file and returns the handle to result.
    fn execute_config_file(config_file_path: *const u8, config_file_path_len: usize) -> u32;
    /// Returns the length of result of executing config file.
    fn result_len(handle: u32) -> usize;
    /// Writes result of executing config file to given buffer.
    fn write_result_to_buffer(handle: u32, buffer: *mut u8, buffer_len: usize) -> usize;
    /// Frees memory allocated for result of executing config file.
    fn free_result(handle: u32);
}

#[cfg(not(target_os = "wasi"))]
unsafe fn execute_config_file(_config_file_path: *const u8, _config_file_path_len: usize) -> u32 {
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

/// Executes given config file and returns the result.
pub fn execute_config(config_file_path: &Path) -> Result<String, ExecuteConfigError> {
    let config_file_path = config_file_path.to_string_lossy();
    let handle = unsafe { execute_config_file(config_file_path.as_ptr(), config_file_path.len()) };
    if handle == 0 {
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
