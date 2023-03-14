use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde_yaml::Value;

use super::{config::ConfigFile, error::ConfigFileError};

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
fn search_graphql_config() -> io::Result<Option<(PathBuf, String)>> {
    for name in CONFIG_NAMES.iter() {
        match fs::read_to_string(name) {
            Ok(buf) => {
                let current_dir = env::current_dir()?;
                let config_file_path = current_dir.join(name);
                return Ok(Some((config_file_path, buf)));
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(None)
}

/// Loads config file. Returns a pair of loaded file name and loaded config
pub fn load_config(
    config_file: Option<&Path>,
) -> Result<Option<(PathBuf, ConfigFile)>, ConfigFileError> {
    let config_source = match config_file {
        Some(path) => fs::read_to_string(path).map(|source| Some((path.to_owned(), source))),
        None => search_graphql_config(),
    }?;

    let parsed = config_source
        .map(|(path, source)| {
            let res: Result<Value, _> = serde_yaml::from_str(&source);
            res.map(|res| (path, res))
        })
        .transpose()?;

    let result = parsed.map(|(path, source)| {
        read_config(source)
            .ok_or_else(|| ConfigFileError::ValidationError(path.clone()))
            .map(|config| (path, config))
    });
    result.transpose()
}

fn read_config(config: Value) -> Option<ConfigFile> {
    let schema = 'schema: {
        let schema = config.get("schema")?;
        if let Some(string) = schema.as_str() {
            break 'schema Some(vec![string.to_owned()]);
        }
        if let Some(seq) = schema.as_sequence() {
            let strs: Option<Vec<String>> = seq
                .iter()
                .map(|value| value.as_str().map(|s| s.to_owned()))
                .collect();
            let strs = strs?;
            break 'schema Some(strs);
        }
        None
    };
    let documents = 'documents: {
        let documents = config.get("documents")?;
        if let Some(string) = documents.as_str() {
            break 'documents Some(vec![string.to_owned()]);
        }
        if let Some(seq) = documents.as_sequence() {
            let strs: Option<Vec<String>> = seq
                .iter()
                .map(|value| value.as_str().map(|s| s.to_owned()))
                .collect();
            let strs = strs?;
            break 'documents Some(strs);
        }
        None
    };
    let extensions = config["extensions"]["nitrogql"].as_mapping();
    let schema_output = extensions
        .and_then(|extensions| extensions["schema-output"].as_str())
        .map(|path| path.into());
    Some(ConfigFile {
        schema,
        documents,
        schema_output,
    })
}
