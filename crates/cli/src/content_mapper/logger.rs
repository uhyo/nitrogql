use std::str::FromStr;

use log::{LevelFilter, Log, Metadata, Record};

/// Logger that writes to stderr, as stdout is used for communication with TypeScript.
struct StderrLogger;

static LOGGER: StderrLogger = StderrLogger;

impl Log for StderrLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }
    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("[{}] {}", record.level(), record.args());
        }
    }
    fn flush(&self) {}
}

/// Initializes the logger. Log level is read from `RUST_LOG` env.
pub fn init_logger() {
    let level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|level| LevelFilter::from_str(&level).ok())
        .unwrap_or(LevelFilter::Error);
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(level);
    }
}
