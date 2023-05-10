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
    #[error("Introspection JSON can only be specified once")]
    IntrospectionOnce,
    #[error("Cannot mix GraphQL and Introspection JSON for schema")]
    MixGraphQLAndIntrospection,
    #[error("Option '{option}' is required for the '{command}' command. ")]
    OptionRequired { option: String, command: String },
    #[error("Cannot emit code including runtime to a .d.ts file.")]
    CannotEmitRuntimeToDts,
    #[error("Failed to calculate source map file name for '{path}'.")]
    FailedToCalculateSourceMapFileName { path: PathBuf },
    #[error("{0}")]
    GlobError(String),
    #[error("Command not successful: {0}")]
    CommandNotSuccessful(String),
}
