#![cfg_attr(target_family = "wasm", no_main)]
mod js_printer;
mod loader;
mod logger;
mod tasks;

use std::{cell::RefCell, slice};

use log::debug;
use nitrogql_config_file::Config;

use crate::logger::StringLogger;

thread_local! {
    /// Loaded config.
    static CONFIG: RefCell<Config> = RefCell::new(Config::default());
    /// Result of last operation.
    static RESULT: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Global set of tasks.
    static TASKS: RefCell<tasks::Tasks> = RefCell::new(tasks::Tasks::new());
}

/// Logger.
static LOGGER: StringLogger = StringLogger::new();

#[cfg(not(target_family = "wasm"))]
fn main() {}

/// Initialize this reactor
#[no_mangle]
pub extern "C" fn init(debug: usize) {
    log::set_logger(&LOGGER).expect("failed to set logger");
    if debug != 0 {
        log::set_max_level(log::LevelFilter::Debug);
    }
}

/// Allocate a string buffer of given size.
///
/// # Safety
/// Caller should guarantee that the contents of returned buffer should be valid UTF-8 strings.
#[no_mangle]
pub extern "C" fn alloc_string(len_bytes: usize) -> *mut u8 {
    let str = Box::new(String::with_capacity(len_bytes));
    let str = Box::leak(str);
    str.as_mut_ptr()
}

/// Frees a string buffer returned by `alloc_string`.
///
/// # Safety
/// `free_string` should only be called with a pointer returned by `alloc_string` with the same value of `len_bytes` argument.
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut u8, len_bytes: usize) {
    let _ = unsafe { String::from_raw_parts(ptr, 0, len_bytes) };
}

/// Loads config from given source. Returns true if successful
#[no_mangle]
pub extern "C" fn load_config(config_file_ptr: *const u8, config_file_len: usize) -> bool {
    let config_file = read_str_ptr(config_file_ptr, config_file_len);
    load_config_impl(&config_file)
}

/// Initiates a task with given filename and source.
/// Returns the task id if successful, otherwise 0.
#[no_mangle]
pub extern "C" fn initiate_task(
    file_name_ptr: *const u8,
    file_name_len: usize,
    input_source_ptr: *const u8,
    input_source_len: usize,
) -> usize {
    debug!(
        "initiate_task {file_name_ptr:?} {file_name_len} {input_source_ptr:?} {input_source_len}"
    );
    let file_name = read_str_ptr(file_name_ptr, file_name_len);
    let input_source = read_str_ptr(input_source_ptr, input_source_len);
    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        match loader::initiate_task(&mut tasks, file_name.into(), input_source) {
            Ok(task_id) => task_id,
            Err(err) => {
                RESULT.with(|cell| cell.replace(Some(format!("{}", err.into_inner()))));
                0
            }
        }
    })
}

/// Get the list of additionally required files for the given task.
/// Returns true if successful.
/// Result is stored in `RESULT` and can be accessed by `get_result_ptr` and `get_result_size`.
#[no_mangle]
pub extern "C" fn get_required_files(task_id: usize) -> bool {
    debug!("get_required_files {task_id}");
    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        match loader::get_required_files(&mut tasks, task_id) {
            Ok(required_files) => {
                let required_files = required_files
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("\n");
                RESULT.with(|cell| cell.replace(Some(required_files)));
                true
            }
            Err(err) => {
                RESULT.with(|cell| cell.replace(Some(format!("{}", err.into_inner()))));
                false
            }
        }
    })
}

/// Load an additional file.
/// Returns true if successful.
#[no_mangle]
pub extern "C" fn load_file(
    task_id: usize,
    file_name_ptr: *const u8,
    file_name_len: usize,
    input_source_ptr: *const u8,
    input_source_len: usize,
) -> bool {
    debug!(
        "load_file {task_id} {file_name_ptr:?} {file_name_len} {input_source_ptr:?} {input_source_len}"
    );
    let file_name = read_str_ptr(file_name_ptr, file_name_len);
    let input_source = read_str_ptr(input_source_ptr, input_source_len);
    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        match loader::load_file(&mut tasks, task_id, file_name.into(), input_source) {
            Ok(_) => true,
            Err(err) => {
                RESULT.with(|cell| cell.replace(Some(format!("{}", err.into_inner()))));
                false
            }
        }
    })
}

/// Converts given GraphQL string to JS.
/// Returns true if successful.
#[no_mangle]
pub extern "C" fn emit_js(task_id: usize) -> bool {
    debug!("convert_to_js {task_id}");
    TASKS.with(|tasks| {
        let tasks = tasks.borrow();
        CONFIG.with(
            |config| match loader::emit_js(&tasks, task_id, &config.borrow()) {
                Ok(js) => {
                    RESULT.with(|cell| cell.replace(Some(js)));
                    true
                }
                Err(err) => {
                    RESULT.with(|cell| cell.replace(Some(format!("{}", err.into_inner()))));
                    false
                }
            },
        )
    })
}

/// Frees the task with given id.
#[no_mangle]
pub extern "C" fn free_task(task_id: usize) {
    debug!("free_task {task_id}");
    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        tasks.remove_task(task_id);
    })
}

/// Gets the pointer to the last result of operation.
#[no_mangle]
pub extern "C" fn get_result_ptr() -> *const u8 {
    RESULT.with(|cell| {
        let r = cell.borrow();
        let s = r.as_ref().unwrap();
        s.as_ptr()
    })
}

/// Gets the size of the last result of operation.
#[no_mangle]
pub extern "C" fn get_result_size() -> usize {
    RESULT.with(|cell| {
        let r = cell.borrow();
        let s = r.as_ref().unwrap();
        s.len()
    })
}

/// Writes the log to the result buffer.
#[no_mangle]
pub extern "C" fn get_log() {
    let log = LOGGER.take_log();
    RESULT.with(|cell| cell.replace(Some(log)));
}

fn load_config_impl(config_file: &str) -> bool {
    let config = nitrogql_config_file::parse_config(config_file);
    match config {
        None => false,
        Some(config) => {
            CONFIG.with(|cell| cell.replace(config));
            true
        }
    }
}

fn read_str_ptr(ptr: *const u8, len: usize) -> String {
    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    String::from_utf8(slice.to_vec()).unwrap()
}
