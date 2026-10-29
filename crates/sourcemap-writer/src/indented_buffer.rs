use std::borrow::BorrowMut;

/// Buffer that indents lines written to it.
/// Shared by the writers, which add their own handling of mappings on top of it.
///
/// A line is indented when its first content is written,
/// so that empty lines are not indented.
pub struct IndentedBuffer<B> {
    buffer: B,
    indent: usize,
    indent_str: String,
    pending_indent: bool,
}

impl<B: BorrowMut<String>> IndentedBuffer<B> {
    pub fn new(buffer: B) -> Self {
        IndentedBuffer {
            buffer,
            indent: 0,
            indent_str: String::new(),
            pending_indent: false,
        }
    }

    /// Returns the content written so far.
    pub fn as_str(&self) -> &str {
        self.buffer.borrow()
    }

    /// Returns the underlying buffer, for writing to it without indentation.
    pub fn buffer_mut(&mut self) -> &mut String {
        self.buffer.borrow_mut()
    }

    pub fn into_buffer(self) -> B {
        self.buffer
    }

    /// Write given chunk.
    pub fn write(&mut self, chunk: &str) {
        self.write_with(chunk, String::push_str);
    }

    /// Write given chunk, with `push_line` writing the content of each line to the buffer.
    pub fn write_with(&mut self, chunk: &str, mut push_line: impl FnMut(&mut String, &str)) {
        for (idx, line) in chunk.split('\n').enumerate() {
            if idx > 0 {
                self.buffer.borrow_mut().push('\n');
                self.pending_indent = true;
            }
            if line.is_empty() {
                continue;
            }
            self.flush_pending_indent();
            push_line(self.buffer.borrow_mut(), line);
        }
    }

    /// Write the indent of the current line if it is not written yet.
    pub fn flush_pending_indent(&mut self) {
        if self.pending_indent {
            self.buffer.borrow_mut().push_str(&self.indent_str);
            self.pending_indent = false;
        }
    }

    /// Increase indent level.
    pub fn indent(&mut self) {
        self.indent += 2;
        self.indent_str = " ".repeat(self.indent);
    }

    /// Decrease indent level.
    pub fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(2);
        self.indent_str = " ".repeat(self.indent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indents_non_empty_lines() {
        let mut buffer = IndentedBuffer::new(String::new());
        buffer.write("a {\n");
        buffer.indent();
        buffer.write("b\n\nc");
        buffer.write("d\n");
        buffer.dedent();
        buffer.write("}\n");
        assert_eq!(buffer.into_buffer(), "a {\n  b\n\n  cd\n}\n");
    }

    #[test]
    fn indent_is_written_with_the_first_content_of_a_line() {
        let mut string = String::new();
        let mut buffer = IndentedBuffer::new(&mut string);
        buffer.indent();
        buffer.write("a\n");
        // The indent of the new line follows the indent level when its content is written.
        buffer.dedent();
        buffer.write("b\n");
        buffer.indent();
        buffer.flush_pending_indent();
        assert_eq!(buffer.as_str(), "a\nb\n  ");
        buffer.write_with("c\nd", |buffer, line| buffer.push_str(&line.to_uppercase()));
        assert_eq!(string, "a\nb\n  C\n  D");
    }
}
