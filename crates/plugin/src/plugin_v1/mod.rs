use std::collections::HashMap;

use nitrogql_ast::{base::Pos, TypeSystemDocument};
use nitrogql_printer::{ts_types::TSType, ResolverTypePrinterOptions};

/// Interface of a naked plugin.
#[allow(unused_variables)]
pub trait PluginV1Beta: std::fmt::Debug {
    /// Name of the plugin.
    fn name(&self) -> &str;
    /// Load schema extensions.
    fn load_schema_extensions(&mut self, extensions: PluginSchemaExtensions) {}
    /// Returns additional schema definition provided by the plugin.
    fn schema_addition(&self) -> Option<String> {
        None
    }
    /// Checks schema.
    fn check_schema(&self, schema: &TypeSystemDocument) -> PluginCheckResult {
        PluginCheckResult::success()
    }
    /// Transforms resolver output types.
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        options: &ResolverTypePrinterOptions,
        base: HashMap<&'src str, TSType>,
    ) -> HashMap<&'src str, TSType> {
        base
    }
    /// Transforms document so that it represents which fields
    /// have resolvers.
    fn transform_document_for_resolvers<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> Option<TypeSystemDocument<'src>> {
        None
    }
    /// Transforms document so that it represents the runtime
    /// schema used by a GraphQL server.
    fn transform_document_for_runtime_server<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> Option<TypeSystemDocument<'src>> {
        None
    }
}

pub struct PluginSchemaExtensions<'a> {
    /// Collection of extensions for type.
    pub type_extensions: &'a HashMap<String, HashMap<String, serde_yaml::Value>>,
}

pub struct PluginCheckResult {
    pub errors: Vec<PluginCheckError>,
}

impl PluginCheckResult {
    pub fn success() -> Self {
        Self { errors: vec![] }
    }
}

pub struct PluginCheckError {
    pub position: Pos,
    pub message: String,
    pub additional_info: Vec<(Pos, String)>,
}
