use serde::Deserialize;

use crate::{parsing_utils::StringOrVecString, Config, GenerateConfig};

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
    Some(Config {
        schema: parsed.schema.map(|s| s.into_vec()).unwrap_or_default(),
        operations: parsed.documents.map(|s| s.into_vec()).unwrap_or_default(),
        generate: parsed
            .extensions
            .and_then(|e| e.nitrogql)
            .and_then(|g| g.generate)
            .unwrap_or_default(),
    })
}
