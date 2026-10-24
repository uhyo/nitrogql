use crate::file_store::{FileKind, FileStore};

/// File name for virtual files added by plugins.
pub const PLUGIN_FILE_NAME: &str = "(plugin)";

pub struct PluginHost<'host> {
    pub file_store: &'host mut FileStore,
}

impl<'host> PluginHost<'host> {
    pub fn new(file_store: &'host mut FileStore) -> Self {
        Self { file_store }
    }
}

impl nitrogql_plugin::PluginHost for PluginHost<'_> {
    fn load_virtual_file(&mut self, content: String) -> &'static str {
        let index = self
            .file_store
            .add_file(PLUGIN_FILE_NAME.into(), content, FileKind::Schema);
        let (_, content, _) = self.file_store.get_file(index).unwrap();
        content
    }
}
