use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("No command specified")]
    NoCommandSpecified,
    #[error("Unknown command '{0}'")]
    UnknownCommand(String),
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Schema file not specified")]
    NoSchemaSpecified,
    #[error("Option '{option}' is required for the '{command}' command. ")]
    OptionRequired { option: String, command: String },
    #[error("Failed to calculate source map file name for '{path}'.")]
    FailedToCalculateSourceMapFileName { path: PathBuf },
    #[error("{0}")]
    GlobError(String),
    #[error("Command not successful: {0}")]
    CommandNotSuccessful(String),
}
