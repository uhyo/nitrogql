use graphql_type_system::Schema;
use nitrogql_ast::{base::Pos, TypeSystemDocument};
use nitrogql_printer::ts_types::TSType;

/// Interface of a naked plugin.
pub trait PluginV1Beta: std::fmt::Debug {
    /// Name of the plugin.
    fn name(&self) -> &str;
    /// Returns additional schema definition provided by the plugin.
    fn schema_addition(&self) -> Option<String>;
    /// Checks schema.
    fn check_schema(&self, schema: &TypeSystemDocument) -> PluginCheckResult;
    /// Transforms resolver output type.
    fn transform_resolver_output_type(
        &self,
        schema: &Schema<&str, Pos>,
        type_name: &str,
        base: TSType,
    ) -> TSType;
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
