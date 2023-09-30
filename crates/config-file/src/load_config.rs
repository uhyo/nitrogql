use log::trace;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::parse_config::parse_config;
use crate::{config::Config, node::load_default_from_js_file};

use super::error::ConfigFileError;

#[derive(Copy, Clone)]
enum LoaderKind {
    Yaml,
    Js,
}

const CONFIG_NAMES: [(&str, LoaderKind); 19] = [
    ("graphql.config.json", LoaderKind::Yaml),
    ("graphql.config.yaml", LoaderKind::Yaml),
    ("graphql.config.yml", LoaderKind::Yaml),
    ("graphql.config.js", LoaderKind::Js),
    ("graphql.config.mjs", LoaderKind::Js),
    ("graphql.config.cjs", LoaderKind::Js),
    ("graphql.config.ts", LoaderKind::Js),
    ("graphql.config.mts", LoaderKind::Js),
    ("graphql.config.cts", LoaderKind::Js),
    (".graphqlrc", LoaderKind::Yaml),
    (".graphqlrc.json", LoaderKind::Yaml),
    (".graphqlrc.yaml", LoaderKind::Yaml),
    (".graphqlrc.yml", LoaderKind::Yaml),
    (".graphqlrc.js", LoaderKind::Js),
    (".graphqlrc.mjs", LoaderKind::Js),
    (".graphqlrc.cjs", LoaderKind::Js),
    (".graphqlrc.ts", LoaderKind::Js),
    (".graphqlrc.mts", LoaderKind::Js),
    (".graphqlrc.cts", LoaderKind::Js),
];

/// searches graphql config and loads it if one is found.
fn search_graphql_config(cwd: &Path) -> io::Result<Option<(PathBuf, String)>> {
    trace!("search_graphql_config from {}", cwd.display());
    for (name, kind) in CONFIG_NAMES.iter() {
        let config_file_path = cwd.join(name);
        match kind {
            LoaderKind::Yaml => {
                match fs::read_to_string(&config_file_path) {
                    Ok(buf) => {
                        trace!("Found config file {}", config_file_path.display());
                        return Ok(Some((config_file_path, buf)));
                    }
                    Err(err) if err.kind() == io::ErrorKind::NotFound => {
                        trace!("Not found: {}", config_file_path.display());
                    }
                    // Maybe a WASI way of expressing file not found error
                    Err(err)
                        if err.to_string().starts_with(
                            "failed to find a pre-opened file descriptor through which",
                        ) =>
                    {
                        trace!("Not found: {}", config_file_path.display());
                    }
                    Err(err) => return Err(err),
                }
            }
            LoaderKind::Js => {
                if config_file_path.try_exists()? {
                    trace!("Found config file {}", config_file_path.display());
                    return load_default_from_js_file(&config_file_path)
                        .map(|buf| Some((config_file_path, buf)));
                } else {
                    trace!("Not found: {}", config_file_path.display());
                }
            }
        }
    }
    Ok(None)
}

/// Loads config file. Returns a pair of loaded file name and loaded config.
/// Config file should follow the GraphQL Config format: https://the-guild.dev/graphql/config/docs
pub fn load_config(
    cwd: &Path,
    config_file: Option<&Path>,
) -> Result<Option<(PathBuf, Config)>, ConfigFileError> {
    let config_source = match config_file {
        Some(path) => {
            let mut path_to_read = cwd.to_owned();
            path_to_read.push(path);
            fs::read_to_string(&path_to_read).map(|source| Some((path_to_read, source)))
        }
        None => search_graphql_config(cwd),
    }?;

    match config_source {
        None => Ok(None),
        Some((path, source)) => parse_config(&source)
            .map(|config| Some((path.clone(), config)))
            .ok_or_else(|| ConfigFileError::Validation(path)),
    }
}
