use crate::plugin_v1::{PluginCheckResult, PluginV1Beta};

/// Plugin that adds a @model directive to the schema.
#[derive(Debug)]
pub struct ModelPlugin {}

impl PluginV1Beta for ModelPlugin {
    fn name(&self) -> &str {
        "nitrogql:model-plugin"
    }
    fn schema_addition(&self) -> Option<String> {
        Some(
            r#"
directive model(
  # TypeScript type of this object. Only applicable for whole objects.
  type: String
) on OBJECT | FIELD_DEFINITION
"#
            .into(),
        )
    }
    fn check_schema(&self, _schema: &str) -> PluginCheckResult {
        PluginCheckResult::success()
    }
}
