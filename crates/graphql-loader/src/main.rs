#![cfg_attr(target_family = "wasm", no_main)]
mod js_printer;

use std::{cell::RefCell, mem::ManuallyDrop, slice};

use js_printer::print_js;
use log::{debug, error};
use nitrogql_config_file::Config;
use nitrogql_parser::{parse_operation_document, ParseError};

thread_local! {
    /// Loaded config.
    static CONFIG: RefCell<Config> = RefCell::new(Config::default());
    /// Result of last operation.
    static RESULT: RefCell<Option<String>> = RefCell::new(None);
}

#[cfg(not(target_family = "wasm"))]
fn main() {}

/// Initialize this reactor
#[no_mangle]
pub extern "C" fn init() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .env()
        .init()
        .unwrap();
}

/// Allocate a string buffer of given size.
///
/// # Safety
/// Caller should guarantee that the contents of returned buffer should be valid UTF-8 strings.
#[no_mangle]
pub extern "C" fn alloc_string(len_bytes: usize) -> *mut u8 {
    let str = String::with_capacity(len_bytes);
    let mut str = ManuallyDrop::new(str);
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
    load_config_impl(config_file)
}

/// Converts given GraphQL string to JS.
/// Returns true if successful.
#[no_mangle]
pub extern "C" fn convert_to_js(source_ptr: *const u8, source_len: usize) -> bool {
    debug!("convert_to_js {source_ptr:?} {source_len}");
    let source = read_str_ptr(source_ptr, source_len);
    match convert_to_js_impl(source) {
        Ok(res) => {
            RESULT.with(|cell| cell.replace(Some(res)));
            true
        }
        Err(err) => {
            error!("{}", err.into_message());
            RESULT.with(|cell| cell.replace(None));
            false
        }
    }
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

fn convert_to_js_impl(source: &str) -> Result<String, ParseError> {
    let document = parse_operation_document(source)?;
    let js = CONFIG.with(|cell| {
        let config = cell.borrow();
        print_js(&document, &config)
    });
    Ok(js)
}

fn load_config_impl(config_file: &str) -> bool {
    let config = nitrogql_config_file::parse_config(config_file);
    match config {
        None => false,
        Some(config) => {
            CONFIG.with(|cell| cell.replace(config));
            debug!("Loaded config from given source");
            true
        }
    }
}

fn read_str_ptr(ptr: *const u8, len: usize) -> &'static str {
    let slice = unsafe { slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(slice).unwrap()
}
