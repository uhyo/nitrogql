use std::path::PathBuf;

use serde_yaml::{Mapping, Value};

use crate::{Config, GenerateConfig, GenerateMode};

/// Parse config file from given string.
/// Returns None if there is a validation error.
pub fn parse_config(source: &str) -> Option<Config> {
    let parsed: Value = serde_yaml::from_str(&source).ok()?;

    read_config(parsed)
}

fn read_config(config: Value) -> Option<Config> {
    let schema = 'schema: {
        let schema = config.get("schema");
        let Some(schema) = schema else {
            break 'schema vec![];
        };
        if let Some(string) = schema.as_str() {
            break 'schema vec![string.to_owned()];
        }
        if let Some(seq) = schema.as_sequence() {
            let strs: Option<Vec<String>> = seq
                .iter()
                .map(|value| value.as_str().map(|s| s.to_owned()))
                .collect();
            let strs = strs?;
            break 'schema strs;
        }
        vec![]
    };
    let documents = 'documents: {
        let documents = config.get("documents");
        let Some(documents) = documents else {
            break 'documents vec![];
        };
        if let Some(string) = documents.as_str() {
            break 'documents vec![string.to_owned()];
        }
        if let Some(seq) = documents.as_sequence() {
            let strs: Option<Vec<String>> = seq
                .iter()
                .map(|value| value.as_str().map(|s| s.to_owned()))
                .collect();
            let strs = strs?;
            break 'documents strs;
        }
        vec![]
    };
    let extensions = config
        .get("extensions")
        .and_then(|e| e.get("nitrogql"))
        .and_then(|e| e.as_mapping());
    let generate = extensions.map(generate_config).unwrap_or_default();
    Some(Config {
        schema,
        operations: documents,
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
        .get("schemaOutput")
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
    if let Some(scalar_types) = generate.get("scalarTypes").and_then(|v| v.as_mapping()) {
        config.scalar_types = scalar_types
            .iter()
            .filter_map(|(key, value)| match (key.as_str(), value.as_str()) {
                (Some(key), Some(value)) => Some((key.to_owned(), value.to_owned())),
                _ => None,
            })
            .collect();
    }

    config
}
