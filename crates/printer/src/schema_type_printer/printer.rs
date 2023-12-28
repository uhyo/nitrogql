use std::collections::HashMap;

use nitrogql_ast::type_system::TypeSystemDocument;
use nitrogql_config_file::{Config, ScalarTypeConfig};
use nitrogql_semantics::ast_to_type_system;
use sourcemap_writer::SourceMapWriter;

use crate::schema::get_builtin_scalar_types;

use super::{
    context::SchemaTypePrinterContext, error::SchemaTypePrinterResult, type_printer::TypePrinter,
};

pub struct SchemaTypePrinterOptions {
    /// Type of each scalar. Provided as raw TypeScript code.
    pub scalar_types: HashMap<String, ScalarTypeConfig>,
    /// Special type name for types that includes schema metadata
    pub schema_metadata_type: String,
    /// Whether to make input nullable fields optional.
    pub input_nullable_field_is_optional: bool,
    /// Whether to emit runtime for generated schema types.
    pub emit_schema_runtime: bool,
}

impl Default for SchemaTypePrinterOptions {
    fn default() -> Self {
        SchemaTypePrinterOptions {
            scalar_types: get_builtin_scalar_types(),
            schema_metadata_type: "__nitrogql_schema".into(),
            input_nullable_field_is_optional: true,
            emit_schema_runtime: false,
        }
    }
}

impl SchemaTypePrinterOptions {
    /// Generate from config.
    pub fn from_config(config: &Config) -> Self {
        let mut result = SchemaTypePrinterOptions {
            emit_schema_runtime: config.generate.emit_schema_runtime,
            input_nullable_field_is_optional: config
                .generate
                .r#type
                .allow_undefined_as_optional_input,
            ..SchemaTypePrinterOptions::default()
        };
        result.scalar_types.extend(
            config
                .generate
                .r#type
                .scalar_types
                .iter()
                .map(|(key, value)| (key.to_owned(), value.clone())),
        );
        result
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

    pub fn print_document(&mut self, document: &TypeSystemDocument) -> SchemaTypePrinterResult<()> {
        let schema = ast_to_type_system(document);
        let context = SchemaTypePrinterContext::new(&self.options, document, &schema);
        document.print_type(&context, self.writer)?;
        Ok(())
    }
}
