use crate::file_store::{FileKind, FileStore};

pub struct PluginHost<'host> {
    pub file_store: &'host mut FileStore,
}

impl<'host> PluginHost<'host> {
    pub fn new(file_store: &'host mut FileStore) -> Self {
        Self { file_store }
    }
}

impl<'host> nitrogql_plugin::PluginHost for PluginHost<'host> {
    fn load_virtual_file(&mut self, content: String) -> &'static str {
        let index = self
            .file_store
            .add_file("(plugin)".into(), content, FileKind::Schema);
        let (_, content, _) = self.file_store.get_file(index).unwrap();
        content
    }
}
