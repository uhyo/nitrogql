use nitrogql_ast::base::HasPos;

use super::writer::SourceMapWriter;

/// Writer that writes given content as a JavaScript string.
pub struct JsStringWriter<'a> {
    pub buffer: &'a mut String,
    indent: usize,
    indent_str: String,
    has_indent_flag: bool,
}

impl JsStringWriter<'_> {
    pub fn new(buffer: &mut String) -> JsStringWriter {
        buffer.push_str("`\n");
        JsStringWriter {
            buffer,
            indent: 0,
            indent_str: String::new(),
            has_indent_flag: false,
        }
    }
}

impl Drop for JsStringWriter<'_> {
    fn drop(&mut self) {
        self.buffer.push_str("\n`");
    }
}

impl SourceMapWriter for JsStringWriter<'_> {
    fn write(&mut self, chunk: &str) {
        for (idx, line) in chunk.split('\n').enumerate() {
            if idx > 0 {
                self.buffer.push('\n');
                self.has_indent_flag = true;
            }
            if line.is_empty() {
                continue;
            }
            if self.has_indent_flag {
                self.buffer.push_str(&self.indent_str);
                self.has_indent_flag = false;
            }
            // self.buffer.push_str(line);
            let mut dollar_flag = false;
            for c in line.chars() {
                match c {
                    '\\' => self.buffer.push_str("\\\\"),
                    '$' => self.buffer.push('$'),
                    '`' => self.buffer.push_str("\\`"),
                    '{' => {
                        if dollar_flag {
                            self.buffer.push_str("\\{");
                        } else {
                            self.buffer.push('{');
                        }
                    }
                    _ => self.buffer.push(c),
                }
                dollar_flag = c == '$';
            }
        }
    }
    fn write_for(&mut self, chunk: &str, _node: &impl HasPos) {
        self.write(chunk);
    }
    fn indent(&mut self) {
        self.indent += 2;
        self.indent_str = " ".repeat(self.indent);
    }
    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
        self.indent_str = " ".repeat(self.indent);
    }
}
