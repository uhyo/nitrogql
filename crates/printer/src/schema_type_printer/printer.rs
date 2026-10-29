use std::collections::HashMap;

use nitrogql_ast::type_system::{TypeDefinition, TypeSystemDefinition, TypeSystemDocument};
use nitrogql_config_file::{Config, ScalarTypeConfig, TypeTarget};
use nitrogql_semantics::ast_to_type_system;
use sourcemap_writer::SourceMapWriter;

use crate::{schema::get_builtin_scalar_types, ts_types::TSType};

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
        self.print_prelude(document);

        for target in [
            TypeTarget::OperationInput,
            TypeTarget::OperationOutput,
            TypeTarget::ResolverInput,
            TypeTarget::ResolverOutput,
        ] {
            writeln!(self.writer, "export declare namespace {target} {{");
            self.writer.indent();
            let context = SchemaTypePrinterContext::new(&self.options, document, &schema, target);
            for def in document.definitions.iter() {
                def.print_type(&context, self.writer)?;
                self.writer.write("\n");
            }
            self.writer.dedent();
            writeln!(self.writer, "}}\n");
        }

        let context = SchemaTypePrinterContext::new(
            &self.options,
            document,
            &schema,
            // target is dummy
            TypeTarget::OperationOutput,
        );
        for def in document.definitions.iter() {
            def.print_representative(&context, self.writer)?;
            self.writer.write("\n");
        }

        Ok(())
    }

    fn print_prelude(&mut self, document: &TypeSystemDocument) {
        self.writer.write("export type ");
        self.writer.write(&self.options.schema_metadata_type);
        self.writer.write(" = ");
        let schema_metadata_type = get_schema_metadata_type(document);
        schema_metadata_type.print_type(self.writer);
        self.writer.write(";\n\n");
        // Print utility types
        self.writer.write(
            "type __Beautify<Obj> = { [K in keyof Obj]: Obj[K] } & {};
export type __SelectionSet<Orig, Obj, Others> =
  __Beautify<Pick<{
    [K in keyof Orig]: Obj extends { [P in K]?: infer V } ? V : unknown
  }, Extract<keyof Orig, keyof Obj>> & Others>;

",
        );
    }
}
fn get_schema_metadata_type(document: &TypeSystemDocument) -> TSType {
    let schema_definition = document.definitions.iter().find_map(|def| match def {
        TypeSystemDefinition::SchemaDefinition(def) => Some(def),
        _ => None,
    });
    if let Some(schema_def) = schema_definition {
        return TSType::object(schema_def.definitions.iter().map(|(op, ty)| {
            (
                op.as_str(),
                TSType::TypeVariable(ty.into()),
                schema_def.description.as_ref().map(|d| d.value.clone()),
            )
        }));
    }
    // If there is no schema definition, use default root type names.
    let mut operations = vec![];
    for d in document.definitions.iter() {
        let TypeSystemDefinition::TypeDefinition(def) = d else {
            continue;
        };
        let TypeDefinition::Object(def) = def else {
            continue;
        };

        match def.name.name {
            "Query" => {
                operations.push(("query", (&def.name).into()));
            }
            "Mutation" => {
                operations.push(("mutation", (&def.name).into()));
            }
            "Subscription" => {
                operations.push(("subscription", (&def.name).into()));
            }
            _ => {}
        }
    }

    TSType::object(
        operations
            .into_iter()
            .map(|(op, ty)| (op, TSType::TypeVariable(ty), None)),
    )
}
