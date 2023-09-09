use nitrogql_plugin::{ModelPlugin, Plugin, PluginV1Beta};

use crate::error::CliError;

/// Load plugins by name.
pub fn load_plugins<'host, S: AsRef<str>>(plugins: &[S]) -> Result<Vec<Plugin<'host>>, CliError> {
    plugins
        .iter()
        .map(|plugin| {
            let p: Result<Box<dyn PluginV1Beta>, _> = match plugin.as_ref() {
                "nitrogql:model-plugin" => Ok(Box::new(ModelPlugin {})),
                _ => Err(CliError::CannotLoadPlugin(plugin.as_ref().to_string())),
            };
            p.map(Plugin::new)
        })
        .collect()
}
