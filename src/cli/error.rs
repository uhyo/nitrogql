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
    #[error("{0}")]
    GlobError(String),
    #[error("Command not successful: {0}")]
    CommandNotSuccessful(String),
}
