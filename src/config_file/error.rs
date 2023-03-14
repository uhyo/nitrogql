use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigFileError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error(transparent)]
    YAMLError(#[from] serde_yaml::Error),
    #[error("Cannot load config file '{0}': validation error")]
    ValidationError(PathBuf),
}
