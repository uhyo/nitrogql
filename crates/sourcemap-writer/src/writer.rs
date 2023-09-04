use nitrogql_ast::base::HasPos;

pub trait SourceMapWriter {
    /// Write given chunk without mapping to source.
    fn write(&mut self, chunk: &str);
    /// Write given chunk with a mapping to source.
    fn write_for(&mut self, chunk: &str, node: &impl HasPos);
    /// Increase indent level.
    fn indent(&mut self);
    /// Decrease indent level.
    fn dedent(&mut self);

    /// Compatible with `write!` macro.
    fn write_fmt(&mut self, args: std::fmt::Arguments) {
        self.write(&args.to_string());
    }
}
