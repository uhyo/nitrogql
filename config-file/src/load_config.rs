use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::config::ConfigFile;
use crate::parse_config::parse_config;

use super::error::ConfigFileError;

const CONFIG_NAMES: [&str; 7] = [
    "graphql.config.json",
    "graphql.config.yaml",
    "graphql.config.yml",
    ".graphqlrc",
    ".graphqlrc.json",
    ".graphqlrc.yaml",
    ".graphqlrc.yml",
];

/// searches graphql config and loads it if one is found.
fn search_graphql_config(cwd: &Path) -> io::Result<Option<(PathBuf, String)>> {
    for name in CONFIG_NAMES.iter() {
        let config_file_path = cwd.join(name);
        match fs::read_to_string(&config_file_path) {
            Ok(buf) => {
                return Ok(Some((config_file_path, buf)));
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            // Maybe a WASI way of expressing file not found error
            Err(err)
                if err
                    .to_string()
                    .starts_with("failed to find a pre-opened file descriptor through which") => {}
            Err(err) => return Err(err),
        }
    }
    Ok(None)
}

/// Loads config file. Returns a pair of loaded file name and loaded config.
/// Config file should follow the GraphQL Config format: https://the-guild.dev/graphql/config/docs
pub fn load_config(
    cwd: &Path,
    config_file: Option<&Path>,
) -> Result<Option<(PathBuf, ConfigFile)>, ConfigFileError> {
    let config_source = match config_file {
        Some(path) => fs::read_to_string(path).map(|source| Some((path.to_owned(), source))),
        None => search_graphql_config(cwd),
    }?;

    match config_source {
        None => Ok(None),
        Some((path, source)) => parse_config(&source)
            .map(|config| Some((path.clone(), config)))
            .ok_or_else(|| ConfigFileError::ValidationError(path)),
    }
}
