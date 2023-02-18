use super::has_pos::HasPos;

pub trait SourceMapWriter {
    fn write(&mut self, chunk: &str);
    fn write_for(&mut self, chunk: &str, node: &impl HasPos);
    fn indent(&mut self);
    fn dedent(&mut self);
}
