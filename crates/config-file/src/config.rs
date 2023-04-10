use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, Default)]
pub struct Config {
    /// Path(s) to schema definition files.
    pub schema: Vec<String>,
    /// Path(s) to operation definition files.
    pub operations: Vec<String>,
    // extensions
    pub generate: GenerateConfig,
}

/// Config related to the 'generate' command.
#[derive(Debug)]
pub struct GenerateConfig {
    /// Mode of generation.
    pub mode: GenerateMode,
    /// Output file path for schema.
    pub schema_output: Option<PathBuf>,
    /// Mapping from GraphQL scalar types to TypeScript types.
    pub scalar_types: HashMap<String, String>,
    /// Whether operation is exported as a default export.
    /// Effective only when a document contains only one operation.
    pub default_export_for_operation: bool,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        GenerateConfig {
            mode: Default::default(),
            schema_output: None,
            scalar_types: Default::default(),
            default_export_for_operation: true,
        }
    }
}

/// Mode of code generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum GenerateMode {
    /// To be used with a webpack loader for .graphql files, emits .d.graphql.ts files that are supported by TS 5.0 and later
    #[default]
    WithLoaderTS5_0,
    /// To be used with a webpack loader for .graphql files, emits .d.graphql.ts files that are supported by TS 4.0
    WithLoaderTS4_0,
    /// To be used standalone. Emits .graphql.ts that are supported by TS 4.0
    StandaloneTS4_0,
}

pub struct FromStrError;

impl FromStr for GenerateMode {
    type Err = FromStrError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "with-loader-ts-5.0" => Ok(GenerateMode::WithLoaderTS5_0),
            "with-loader-ts-4.0" => Ok(GenerateMode::WithLoaderTS4_0),
            "standalone-ts-4.0" => Ok(GenerateMode::StandaloneTS4_0),
            _ => Err(FromStrError),
        }
    }
}
