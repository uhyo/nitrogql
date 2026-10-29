use nitrogql_ast::base::HasPos;

use super::{indented_buffer::IndentedBuffer, writer::SourceMapWriter};

pub struct JustWriter<'a> {
    buffer: IndentedBuffer<&'a mut String>,
}

impl JustWriter<'_> {
    pub fn new(buffer: &mut String) -> JustWriter<'_> {
        JustWriter {
            buffer: IndentedBuffer::new(buffer),
        }
    }
}

impl SourceMapWriter for JustWriter<'_> {
    fn write(&mut self, chunk: &str) {
        self.buffer.write(chunk);
    }
    fn write_for(&mut self, chunk: &str, _node: &impl HasPos) {
        self.write(chunk);
    }
    fn indent(&mut self) {
        self.buffer.indent();
    }
    fn dedent(&mut self) {
        self.buffer.dedent();
    }
}
