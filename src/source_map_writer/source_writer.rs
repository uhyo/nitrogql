use crate::graphql_parser::ast::base::HasPos;

use self::{mapping_writer::MappingWriter, name_mapper::NameMapper};

use super::writer::SourceMapWriter;

mod mapping_writer;
mod name_mapper;
mod utf16_len;

use json_writer::JSONObjectWriter;
use utf16_len::utf16_len;

pub struct SourceWriter {
    buffer: String,
    mapping: MappingWriter,
    name_mapper: NameMapper,
    indent: usize,
    indent_str: String,
    has_indent_flag: bool,

    /// Line number in generated source. (0-based)
    current_line: usize,
    /// Column number in generated source. (0-based)
    current_column: usize,
}

impl SourceWriter {
    pub fn new<'a>() -> Self {
        SourceWriter {
            buffer: String::new(),
            mapping: MappingWriter::new(),
            name_mapper: NameMapper::new(),
            indent: 0,
            indent_str: String::new(),
            has_indent_flag: false,
            current_line: 0,
            current_column: 0,
        }
    }

    pub fn into_buffers(self) -> SourceWriterBuffers {
        SourceWriterBuffers {
            buffer: self.buffer,
            source_map: self.mapping.into_buffer(),
            names: self.name_mapper.into_names(),
        }
    }
}

pub struct SourceWriterBuffers {
    pub buffer: String,
    pub source_map: String,
    pub names: Vec<String>,
}

impl SourceMapWriter for SourceWriter {
    fn write(&mut self, chunk: &str) {
        for (idx, line) in chunk.split('\n').enumerate() {
            if idx > 0 {
                self.buffer.push('\n');
                self.has_indent_flag = true;
                self.current_line += 1;
                self.current_column = 0;
            }
            if line.is_empty() {
                continue;
            }
            if self.has_indent_flag {
                self.buffer.push_str(&self.indent_str);
                self.current_column += self.indent;
                self.has_indent_flag = false;
            }
            self.buffer.push_str(line);
            self.current_column += utf16_len(line);
        }
    }
    fn write_for(&mut self, chunk: &str, node: &impl HasPos) {
        let original_pos = node.position();
        let original_name = node.name().map(|name| self.name_mapper.map_name(name));
        self.mapping.add_entry(
            self.current_line,
            self.current_column,
            original_pos.line,
            original_pos.column,
            original_name,
        );
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

pub fn print_source_map_json(
    // Name of generated file
    file: &str,
    // Name of source file
    source_files: &[&str],
    names: &[String],
    source_map: &str,
    buffer: &mut String,
) {
    let mut json_writer = JSONObjectWriter::new(buffer);
    json_writer.value("version", 3);
    json_writer.value("file", file);
    json_writer.value("sourceRoot", "");
    json_writer.value("sources", source_files);
    json_writer.value("names", names);
    json_writer.value("mappings", source_map);
}
