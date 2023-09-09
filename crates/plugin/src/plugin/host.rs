/// Representation of the host application of the plugin.
pub trait PluginHost<'src> {
    /// Load given string as a virtual file.
    fn load_virtual_file(&mut self, content: String) -> &'src str;
}
