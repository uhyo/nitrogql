use std::{collections::HashMap, path::PathBuf, str::FromStr};

use serde::Deserialize;

use crate::parsing_utils::deserialize_fromstr;

#[derive(Debug, Default)]
pub struct Config {
    /// Path(s) to schema definition files.
    pub schema: Vec<String>,
    /// Path(s) to operation definition files.
    pub operations: Vec<String>,
    // extensions
    /// List of plugins.
    pub plugins: Vec<String>,
    pub generate: GenerateConfig,
}

/// Config related to the 'generate' command.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GenerateConfig {
    /// Mode of generation.
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub mode: GenerateMode,
    /// Output file path for schema.
    pub schema_output: Option<PathBuf>,
    /// Output file path for processed GraphQL sources.
    pub server_graphql_output: Option<PathBuf>,
    /// Output file path for resolvers.
    pub resolvers_output: Option<PathBuf>,
    /// Module specifier for import schema types from operations.
    /// Defaults to relative paths.
    pub schema_module_specifier: Option<String>,
    /// Config related to generated types.
    pub r#type: GenerateTypeConfig,
    /// Config related to generated names.
    pub name: GenerateNameConfig,
    /// Config related to exporting generated names.
    pub export: GenerateExportConfig,
    /// Whether to emit runtime for generated schema types.
    pub emit_schema_runtime: bool,
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

/// Config related to generated types.
#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GenerateTypeConfig {
    /// Type of scalars.
    pub scalar_types: HashMap<String, String>,
    /// Whether to allow undefined as input value
    /// for nullable input fields.
    pub allow_undefined_as_optional_input: bool,
}

impl Default for GenerateTypeConfig {
    fn default() -> Self {
        Self {
            scalar_types: HashMap::new(),
            allow_undefined_as_optional_input: true,
        }
    }
}

/// Config related to names of generated variables and types.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GenerateNameConfig {
    /// Suffix for type of operation result.
    pub operation_result_type_suffix: Option<String>,
    /// Suffix for type of variables for an operation.
    pub variables_type_suffix: Option<String>,
    /// Suffix for type of fragment.
    pub fragment_type_suffix: Option<String>,
    /// Whether operation name should be capitalized.
    pub capitalize_operation_names: Option<bool>,
    /// Suffix for variable of query.
    pub query_variable_suffix: Option<String>,
    /// Suffix for variable of mutation.
    pub mutation_variable_suffix: Option<String>,
    /// Suffix for variable of subscription.
    pub subscription_variable_suffix: Option<String>,
}

/// Config related to exported names.
#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GenerateExportConfig {
    /// Whether operation is exported as a default export.
    /// Effective only when a document contains only one operation.
    pub default_export_for_operation: bool,
    /// Whether operation result type is exported.
    pub operation_result_type: bool,
    /// Whether variables type is exported.
    pub variables_type: bool,
}

impl Default for GenerateExportConfig {
    fn default() -> Self {
        Self {
            default_export_for_operation: true,
            operation_result_type: false,
            variables_type: false,
        }
    }
}
