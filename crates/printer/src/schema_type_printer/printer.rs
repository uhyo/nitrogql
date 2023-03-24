use std::collections::HashMap;

use graphql_type_system::Schema;
use nitrogql_ast::{base::Pos, type_system::TypeSystemDocument};
use nitrogql_semantics::ast_to_type_system;
use sourcemap_writer::SourceMapWriter;

use super::{error::SchemaTypePrinterResult, type_printer::TypePrinter};

pub struct SchemaTypePrinterOptions {
    /// Type of each scalar. Provided as raw TypeScript code.
    pub scalar_types: HashMap<String, String>,
    /// Special type name for types that includes schema metadata
    pub schema_metadata_type: String,
    /// Whether to make input nullable fields optional.
    pub input_nullable_field_is_optional: bool,
}

impl Default for SchemaTypePrinterOptions {
    fn default() -> Self {
        SchemaTypePrinterOptions {
            scalar_types: get_builtin_scalar_types(),
            schema_metadata_type: "__nitrogql_schema".into(),
            input_nullable_field_is_optional: true,
        }
    }
}

pub struct SchemaTypePrinterContext<'src> {
    pub options: &'src SchemaTypePrinterOptions,
    pub document: &'src TypeSystemDocument<'src>,
    pub schema: &'src Schema<&'src str, Pos>,
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

    pub fn print_document(&mut self, document: &TypeSystemDocument) -> SchemaTypePrinterResult<()> {
        let schema = ast_to_type_system(document);
        let context = SchemaTypePrinterContext {
            options: &self.options,
            document,
            schema: &schema,
        };
        document.print_type(&context, self.writer)?;
        Ok(())
    }
}

/// Generates scalar definitions for built-in scalars.
fn get_builtin_scalar_types() -> HashMap<String, String> {
    vec![
        ("ID".into(), "string".into()),
        ("String".into(), "string".into()),
        ("Int".into(), "number".into()),
        ("Float".into(), "number".into()),
        ("Boolean".into(), "boolean".into()),
    ]
    .into_iter()
    .collect()
}
