use crate::ast::base::HasPos;

use super::writer::SourceMapWriter;

pub struct JustWriter<'a> {
    pub buffer: &'a mut String,
    indent: usize,
    indent_str: String,
    has_indent_flag: bool,
}

impl JustWriter<'_> {
    pub fn new<'a>(buffer: &'a mut String) -> JustWriter<'a> {
        JustWriter {
            buffer,
            indent: 0,
            indent_str: String::new(),
            has_indent_flag: false,
        }
    }
}

impl SourceMapWriter for JustWriter<'_> {
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
            self.buffer.push_str(line);
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
