use nitrogql_ast::base::Pos;

/// Interface of a plugin.
pub trait PluginV1Beta {
    /// Name of the plugin.
    fn name(&self) -> &str;
    /// Returns additional schema definition provided by the plugin.
    fn schema_addition(&self) -> Option<String>;
    /// Checks schema.
    fn check_schema(&self, schema: &str) -> PluginCheckResult;
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
