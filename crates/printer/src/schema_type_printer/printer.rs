use std::{borrow::Cow, collections::HashMap};

use graphql_type_system::Schema;
use nitrogql_ast::{base::Pos, type_system::TypeSystemDocument};
use nitrogql_config_file::Config;
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
                .map(|(key, value)| (key.to_owned(), value.to_owned())),
        );
        result
    }
}

pub struct SchemaTypePrinterContext<'src> {
    pub options: &'src SchemaTypePrinterOptions,
    pub document: &'src TypeSystemDocument<'src>,
    pub schema: &'src Schema<Cow<'src, str>, Pos>,
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
