use std::collections::HashMap;

use nitrogql_ast::{base::Pos, TypeSystemDocument};
use nitrogql_printer::{ts_types::TSType, ResolverTypePrinterOptions};

/// Interface of a naked plugin.
pub trait PluginV1Beta: std::fmt::Debug {
    /// Name of the plugin.
    fn name(&self) -> &str;
    /// Returns additional schema definition provided by the plugin.
    fn schema_addition(&self) -> Option<String>;
    /// Checks schema.
    fn check_schema(&self, schema: &TypeSystemDocument) -> PluginCheckResult;
    /// Transforms resolver output types.
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        options: &ResolverTypePrinterOptions,
        base: HashMap<&'src str, TSType>,
    ) -> HashMap<&'src str, TSType>;
    /// Transforms document so that it represents which fields
    /// have resolvers.
    fn transform_document_for_resolvers<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> TypeSystemDocument<'src>;
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
