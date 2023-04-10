use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigFileError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("Error loading config file: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Cannot load config file '{0}': validation error")]
    Validation(PathBuf),
}
