use std::collections::HashMap;

use crate::{graphql_parser::ast::TypeSystemDocument, source_map_writer::writer::SourceMapWriter};

use super::type_printer::TypePrinter;

pub struct SchemaTypePrinterOptions {
    /// Type of each scalar. Provided as raw TypeScript code.
    pub scalar_types: HashMap<String, String>,
    /// Special type name for types that includes schema metadata
    pub schema_metadata_type: String,
}

impl Default for SchemaTypePrinterOptions {
    fn default() -> Self {
        SchemaTypePrinterOptions {
            scalar_types: HashMap::new(),
            schema_metadata_type: "__nitrogql_schema".into(),
        }
    }
}

pub struct SchemaTypePrinter<'a, Writer: SourceMapWriter> {
    options: SchemaTypePrinterOptions,
    writer: &'a mut Writer,
}

impl<'a, Writer> SchemaTypePrinter<'a, Writer>
where
    Writer: SourceMapWriter,
{
    pub fn new(options: SchemaTypePrinterOptions, writer: &'a mut Writer) -> Self {
        SchemaTypePrinter { options, writer }
    }

    pub fn print_document(&mut self, document: &TypeSystemDocument) {
        document.print_type(&self.options, self.writer);
    }
}
