/// Representation of the host application of the plugin.
pub trait PluginHost {
    /// Load given string as a virtual file.
    /// Returns the loaded content and the index of the new file.
    fn load_virtual_file(&mut self, content: String) -> (&'static str, usize);
}
