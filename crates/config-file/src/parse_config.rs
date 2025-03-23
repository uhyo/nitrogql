use serde::Deserialize;

use crate::{Config, GenerateConfig, parsing_utils::StringOrVecString};

#[derive(Deserialize)]
struct ConfigParser {
    schema: Option<StringOrVecString>,
    documents: Option<StringOrVecString>,
    extensions: Option<Extensions>,
}

#[derive(Deserialize)]
struct Extensions {
    nitrogql: Option<NitrogqlConfigParser>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct NitrogqlConfigParser {
    plugins: Vec<String>,
    generate: Option<GenerateConfig>,
}

/// Parse config file from given string.
/// Returns None if there is a validation error.
pub fn parse_config(source: &str) -> Option<Config> {
    let parsed: ConfigParser = serde_yaml::from_str(source).unwrap();
    let ConfigParser {
        schema,
        documents,
        extensions,
    } = parsed;
    let nitrogql = extensions.and_then(|e| e.nitrogql);
    let (plugins, generate) = nitrogql
        .map(|n| (n.plugins, n.generate.unwrap_or_default()))
        .unwrap_or_default();
    Some(Config {
        schema: schema.map(|s| s.into_vec()).unwrap_or_default(),
        operations: documents.map(|s| s.into_vec()).unwrap_or_default(),
        plugins,
        generate,
    })
}
