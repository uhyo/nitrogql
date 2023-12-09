use std::sync::RwLock;

pub struct StringLogger {
    log: RwLock<String>,
}

impl StringLogger {
    /// Creates a new logger.
    pub const fn new() -> Self {
        Self {
            log: RwLock::new(String::new()),
        }
    }
    /// Returns the log.
    /// Log is cleared after this call.
    pub fn take_log(&self) -> String {
        let mut log = self.log.write().expect("failed to write to log");
        std::mem::take(&mut log)
    }
}

impl log::Log for StringLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        self.log
            .write()
            .expect("failed to write to log")
            .push_str(&format!("{} - {}\n", record.level(), record.args()));
    }
    fn flush(&self) {
        // do nothing
    }
}
