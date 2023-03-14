use std::path::PathBuf;

#[derive(Debug)]
pub struct ConfigFile {
    /// Path(s) to schema definition files.
    pub schema: Option<Vec<String>>,
    /// Path(s) to operation definition files.
    pub documents: Option<Vec<String>>,
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
    /// Whether operation is exported as a default export.
    /// Effective only when a document contains only one operation.
    pub default_export_for_operation: bool,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        GenerateConfig {
            mode: GenerateMode::default(),
            schema_output: None,
            default_export_for_operation: true,
        }
    }
}

/// Mode of code generation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GenerateMode {
    /// To be used with a webpack loader for .graphql files, emits .d.graphql.ts files that are supported by TS 5.0 and later
    WithLoaderTS5_0,
    /// To be used with a webpack loader for .graphql files, emits .d.graphql.ts files that are supported by TS 4.0
    WithLoaderTS4_0,
}

impl GenerateMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            GenerateMode::WithLoaderTS5_0 => "with-loader-ts-5.0",
            GenerateMode::WithLoaderTS4_0 => "with-loader-ts-4.0",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "with-loader-ts-5.0" => Some(GenerateMode::WithLoaderTS5_0),
            "with-loader-ts-4.0" => Some(GenerateMode::WithLoaderTS4_0),
            _ => None,
        }
    }
}

impl Default for GenerateMode {
    fn default() -> Self {
        GenerateMode::WithLoaderTS5_0
    }
}
