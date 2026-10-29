use nitrogql_ast::base::HasPos;

use super::{indented_buffer::IndentedBuffer, writer::SourceMapWriter};

/// Writer that writes given content as a JavaScript string.
pub struct JsStringWriter<'a> {
    buffer: IndentedBuffer<&'a mut String>,
}

impl JsStringWriter<'_> {
    pub fn new(buffer: &mut String) -> JsStringWriter<'_> {
        buffer.push_str("`\n");
        JsStringWriter {
            buffer: IndentedBuffer::new(buffer),
        }
    }
}

impl Drop for JsStringWriter<'_> {
    fn drop(&mut self) {
        self.buffer.buffer_mut().push('`');
    }
}

impl SourceMapWriter for JsStringWriter<'_> {
    fn write(&mut self, chunk: &str) {
        self.buffer.write_with(chunk, |buffer, line| {
            let mut dollar_flag = false;
            for c in line.chars() {
                match c {
                    '\\' => buffer.push_str("\\\\"),
                    '$' => buffer.push('$'),
                    '`' => buffer.push_str("\\`"),
                    '{' => {
                        if dollar_flag {
                            buffer.push_str("\\{");
                        } else {
                            buffer.push('{');
                        }
                    }
                    _ => buffer.push(c),
                }
                dollar_flag = c == '$';
            }
        });
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
