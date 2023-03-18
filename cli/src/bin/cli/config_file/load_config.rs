use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde_yaml::{Mapping, Value};

use crate::cli::cwd::get_cwd;

use super::{
    config::{ConfigFile, GenerateConfig, GenerateMode},
    error::ConfigFileError,
};

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
        let config_file_path = get_cwd()?.join(name);
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
        let schema = config.get("schema");
        let Some(schema) = schema else {
            break 'schema None;
        };
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
        let documents = config.get("documents");
        let Some(documents) = documents else {
            break 'documents None;
        };
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
    let extensions = config
        .get("extensions")
        .and_then(|e| e.get("nitrogql"))
        .and_then(|e| e.as_mapping());
    let generate = extensions.map(generate_config).unwrap_or_default();
    Some(ConfigFile {
        schema,
        documents,
        generate,
    })
}

/// Reads extensions.generate config.
fn generate_config(extensions: &Mapping) -> GenerateConfig {
    let mut config = GenerateConfig::default();
    let Some(generate) = extensions.get("generate") else {
        return config;
    };

    if let Some(schema_output) = generate
        .get("schema-output")
        .and_then(|path| path.as_str())
        .map(PathBuf::from)
    {
        config.schema_output = Some(schema_output);
    }
    if let Some(mode) = generate
        .get("mode")
        .and_then(|v| v.as_str())
        .and_then(GenerateMode::from_str)
    {
        config.mode = mode;
    }
    if let Some(default_export_for_operation) = generate
        .get("defaultExportForOperation")
        .and_then(|v| v.as_bool())
    {
        config.default_export_for_operation = default_export_for_operation;
    }

    config
}