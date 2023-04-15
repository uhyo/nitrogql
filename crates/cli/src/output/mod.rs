use std::path::PathBuf;

use json_writer::JSONObjectWriter;
use nitrogql_checker::CheckError;

mod file_kind;

pub use file_kind::{InputFileKind, OutputFileKind};
use nitrogql_error::print_positioned_error;

use crate::file_store::FileStore;

/// Struct that keeps track of all outputs.
pub struct CliOutput {
    commands_run: Vec<String>,
    command_error: Option<(Option<String>, String)>,
    check_errors: Vec<(file_kind::InputFileKind, CheckError)>,
    generated_files: Vec<(file_kind::OutputFileKind, PathBuf)>,
}

impl CliOutput {
    /// Create an empty set.
    pub fn new() -> Self {
        Self {
            commands_run: Vec::new(),
            command_error: None,
            check_errors: Vec::new(),
            generated_files: Vec::new(),
        }
    }

    /// Indicates that a command is run.
    pub fn command_run(&mut self, command_name: String) {
        self.commands_run.push(command_name);
    }

    /// Add a command error.
    pub fn command_error(&mut self, command_name: Option<String>, error: String) {
        self.command_error = Some((command_name, error));
    }

    /// Add a check error.
    pub fn check_error(&mut self, kind: InputFileKind, error: CheckError) {
        self.check_errors.push((kind, error));
    }

    /// Add a generated file.
    pub fn generated_file(&mut self, kind: OutputFileKind, path: PathBuf) {
        self.generated_files.push((kind, path));
    }

    /// Output for human consumption.
    pub fn human_output(self, file_store: &FileStore) {
        if !self.check_errors.is_empty() {
            let (schema_errors, operation_errors): (Vec<_>, Vec<_>) = self
                .check_errors
                .into_iter()
                .partition(|(kind, _)| match kind {
                    InputFileKind::Schema => true,
                    InputFileKind::Operation => false,
                });
            if !schema_errors.is_empty() {
                eprintln!(
                    "Found {} error{} in schema:",
                    schema_errors.len(),
                    if schema_errors.len() > 1 { "s" } else { "" }
                );
                for (_, error) in schema_errors {
                    eprintln!("{}", print_positioned_error(&error.into(), file_store));
                }
                eprintln!();
            }
            if !operation_errors.is_empty() {
                eprintln!(
                    "Found {} error{} in operations:",
                    operation_errors.len(),
                    if operation_errors.len() > 1 { "s" } else { "" }
                );
                for (_, error) in operation_errors {
                    eprintln!("{}", print_positioned_error(&error.into(), file_store));
                }
                eprintln!();
            }
        }
        if let Some((command_name, error)) = self.command_error {
            match command_name {
                Some(command_name) => eprintln!("Error in command '{command_name}':\n{error}"),
                None => eprintln!("Error:\n{error}"),
            }
        }
    }

    /// Output with JSON format.
    pub fn json_output(self, file_store: &FileStore) {
        let mut buffer = String::new();
        let mut writer = JSONObjectWriter::new(&mut buffer);
        if let Some((command, message)) = self.command_error {
            let mut obj = writer.object("error");
            obj.value("command", command.as_ref());
            obj.value("message", &message);
        }
        if self.commands_run.iter().any(|c| c == "check") {
            let mut obj = writer.object("check");
            let mut errors = obj.array("errors");
            for (kind, error) in self.check_errors {
                let file = (!error.position.builtin)
                    .then(|| file_store.get_file(error.position.file))
                    .flatten();
                let mut obj = errors.object();
                obj.value("fileType", &kind.to_string());
                match file {
                    Some((path, _, _)) => {
                        let mut obj = obj.object("file");
                        obj.value("path", &path.to_string_lossy());
                        obj.value("line", error.position.line as u32);
                        obj.value("column", error.position.column as u32);
                    }
                    None => obj.value("file", None::<&bool>),
                }
                obj.value("message", &error.message.to_string());
            }
        }
        if self.commands_run.iter().any(|c| c == "generate") {
            let mut obj = writer.object("generate");
            let mut files = obj.array("files");
            for (kind, path) in self.generated_files {
                let mut obj = files.object();
                obj.value("fileType", &kind.to_string());
                obj.value("path", &path.to_string_lossy());
            }
        }
        writer.end();
        println!("{buffer}");
    }

    /// Output in rdjson format.
    pub fn rdjson_output(self, file_store: &FileStore) {
        let mut buffer = String::new();
        let mut writer = JSONObjectWriter::new(&mut buffer);
        {
            let mut source = writer.object("source");
            source.value("name", "nitrogql");
            source.value("url", "https://nitrogql.vercel.app/");
        }
        writer.value("severity", "ERROR");
        {
            let mut diagnostics = writer.array("diagnostics");
            for (_, error) in self.check_errors {
                let mut obj = diagnostics.object();
                obj.value("message", &error.message.to_string());
                {
                    let mut location = obj.object("location");
                    let file = (!error.position.builtin)
                        .then(|| file_store.get_file(error.position.file))
                        .flatten();
                    if let Some((path, _, _)) = file {
                        location.value("path", &path.to_string_lossy());
                        let mut range = location.object("range");
                        let mut start = range.object("start");
                        start.value("line", error.position.line as u32 + 1);
                        start.value("column", error.position.column as u32 + 1);
                    }
                }
            }
        }
        writer.end();
        println!("{buffer}");
    }
}

impl Extend<(InputFileKind, CheckError)> for CliOutput {
    fn extend<T: IntoIterator<Item = (InputFileKind, CheckError)>>(&mut self, iter: T) {
        self.check_errors.extend(iter);
    }
}
