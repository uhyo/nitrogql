/// Representation of the host application of the plugin.
/// `'src` is the lifetime of loaded files.
pub trait PluginHost<'src> {
    /// Load given string as a virtual file.
    /// Returns the loaded content and the index of the new file.
    fn load_virtual_file(&mut self, content: String) -> (&'src str, usize);
}
