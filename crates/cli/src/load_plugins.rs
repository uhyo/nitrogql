use nitrogql_plugin::{ModelPlugin, PluginV1Beta};

use crate::error::CliError;

/// Load plugins by name.
pub fn load_plugins<S: AsRef<str>>(plugins: &[S]) -> Result<Vec<Box<dyn PluginV1Beta>>, CliError> {
    plugins
        .iter()
        .map(|plugin| {
            let p: Result<Box<dyn PluginV1Beta>, _> = match plugin.as_ref() {
                "nitrogql:model-plugin" => Ok(Box::new(ModelPlugin {})),
                _ => Err(CliError::CannotLoadPlugin(plugin.as_ref().to_string())),
            };
            p
        })
        .collect()
}
