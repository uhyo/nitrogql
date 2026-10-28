use crate::file_store::{FileKind, FileStore};

/// File name for virtual files added by plugins.
/// This is only for display; plugin files are identified by `FileKind::Plugin`.
pub const PLUGIN_FILE_NAME: &str = "(plugin)";

pub struct PluginHost<'host, 'src> {
    pub file_store: &'host mut FileStore<'src>,
}

impl<'host, 'src> PluginHost<'host, 'src> {
    pub fn new(file_store: &'host mut FileStore<'src>) -> Self {
        Self { file_store }
    }
}

impl<'src> nitrogql_plugin::PluginHost<'src> for PluginHost<'_, 'src> {
    fn load_virtual_file(&mut self, content: String) -> (&'src str, usize) {
        let index = self
            .file_store
            .add_file(PLUGIN_FILE_NAME.into(), content, FileKind::Plugin);
        let (_, content, _) = self.file_store.get_file(index).unwrap();
        (content, index)
    }
}
