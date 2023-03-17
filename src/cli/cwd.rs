//! In WASI environment, we depend on a CWD environment variable for determining current working directory.

use std::path::PathBuf;
use std::{env, io};

#[cfg(target_os = "wasi")]
pub fn get_cwd() -> io::Result<PathBuf> {
    env::var("CWD").map(PathBuf::from).or(env::current_dir())
}

#[cfg(not(target_os = "wasi"))]
pub fn get_cwd() -> io::Result<PathBuf> {
    env::current_dir()
}
