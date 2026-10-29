use nitrogql_ast::base::{HasPos, Pos};

use super::{indented_buffer::IndentedBuffer, writer::SourceMapWriter};

/// Writer that records which spans of generated code correspond to named nodes
/// in the original source.
///
/// Unlike `SourceWriter`, this does not produce a source map; spans are recorded
/// as byte offsets in the generated code so that they can be used for other mapping formats.
pub struct SpanWriter {
    buffer: IndentedBuffer<String>,
    spans: Vec<RecordedSpan>,
}

/// A span of generated code that corresponds to a named node in the original source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedSpan {
    /// Start of span in generated code. (byte offset)
    pub generated_start: usize,
    /// Length of span in generated code. (bytes)
    pub generated_len: usize,
    /// Position of the original node.
    pub original_pos: Pos,
    /// Name of the original node.
    pub original_name: String,
}

impl SpanWriter {
    pub fn new() -> Self {
        SpanWriter {
            buffer: IndentedBuffer::new(String::new()),
            spans: vec![],
        }
    }

    pub fn into_parts(self) -> (String, Vec<RecordedSpan>) {
        (self.buffer.into_buffer(), self.spans)
    }
}

impl Default for SpanWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMapWriter for SpanWriter {
    fn write(&mut self, chunk: &str) {
        self.buffer.write(chunk);
    }
    fn write_for(&mut self, chunk: &str, node: &impl HasPos) {
        let original_pos = node.position();
        let original_name = node.name();
        match original_name {
            Some(original_name) if !original_pos.builtin && !chunk.is_empty() => {
                self.buffer.flush_pending_indent();
                let generated_start = self.buffer.as_str().len();
                self.write(chunk);
                self.spans.push(RecordedSpan {
                    generated_start,
                    generated_len: self.buffer.as_str().len() - generated_start,
                    original_pos: *original_pos,
                    original_name: original_name.to_owned(),
                });
            }
            _ => {
                self.write(chunk);
            }
        }
    }

    fn indent(&mut self) {
        self.buffer.indent();
    }
    fn dedent(&mut self) {
        self.buffer.dedent();
    }
}

#[cfg(test)]
mod tests {
    use nitrogql_ast::base::Ident;

    use super::*;

    #[test]
    fn records_named_spans() {
        let mut writer = SpanWriter::new();
        let ident = Ident {
            name: "Foo",
            position: Pos {
                line: 1,
                column: 2,
                start: 5,
                end: 8,
                file: 0,
                builtin: false,
            },
        };
        writer.write("type ");
        writer.write_for("FooResult", &ident);
        writer.write(" = {\n");
        writer.indent();
        writer.write_for("Foo", &ident);
        writer.write(": 1;\n");
        writer.dedent();
        writer.write("};");
        let (buffer, spans) = writer.into_parts();
        assert_eq!(buffer, "type FooResult = {\n  Foo: 1;\n};");
        assert_eq!(
            spans,
            vec![
                RecordedSpan {
                    generated_start: 5,
                    generated_len: 9,
                    original_pos: ident.position,
                    original_name: "Foo".to_owned(),
                },
                RecordedSpan {
                    generated_start: 21,
                    generated_len: 3,
                    original_pos: ident.position,
                    original_name: "Foo".to_owned(),
                },
            ]
        );
    }
}
