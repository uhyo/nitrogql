use nitrogql_config_file::{Config, GenerateMode};
use nitrogql_utils::clone_into;
use sourcemap_writer::SourceMapWriter;

use crate::{
    jsdoc::print_description,
    operation_base_printer::{
        OperationPrinterVisitor, PrintFragmentContext, PrintOperationContext,
        options::OperationBasePrinterOptions,
    },
    operation_js_printer::{print_fragment_runtime, print_operation_runtime},
};

use super::type_source::OperationTypeSource;

#[derive(Clone, Debug)]
pub struct OperationTypePrinterOptions {
    pub base_options: OperationBasePrinterOptions,
    /// Whether value of variables should be printed.
    pub print_values: bool,
    /// Name of the root TypeScript namespace that contains schema types.
    pub schema_root_namespace: String,
    /// Source of schema type to import from.
    pub schema_source: String,
    /// Source of TypedDocumentNode to import from.
    pub typed_document_node_source: String,
    /// Suffix for type of variables for an operation.
    pub variables_type_suffix: String,
    /// Suffix for type of operation result.
    pub operation_result_type_suffix: String,
    /// Suffix for type of fragment.
    pub fragment_type_suffix: String,
    /// Whether to allow undefined as input value.
    pub allow_undefined_as_optional_input: bool,
}

impl Default for OperationTypePrinterOptions {
    fn default() -> Self {
        Self {
            base_options: OperationBasePrinterOptions::default(),
            print_values: false,
            schema_root_namespace: "Schema".to_owned(),
            schema_source: "".to_owned(),
            typed_document_node_source: "@graphql-typed-document-node/core".to_owned(),
            variables_type_suffix: "Variables".to_owned(),
            operation_result_type_suffix: "Result".to_owned(),
            fragment_type_suffix: "".to_owned(),
            allow_undefined_as_optional_input: true,
        }
    }
}

impl OperationTypePrinterOptions {
    /// Generate options from config.
    pub fn from_config(config: &Config) -> Self {
        let mut result = Self {
            base_options: OperationBasePrinterOptions::from_config(config),
            ..Self::default()
        };
        if config.generate.mode == GenerateMode::StandaloneTS4_0 {
            result.print_values = true;
        }
        clone_into(
            &config.generate.name.operation_result_type_suffix,
            &mut result.operation_result_type_suffix,
        );
        clone_into(
            &config.generate.name.variables_type_suffix,
            &mut result.variables_type_suffix,
        );
        clone_into(
            &config.generate.name.fragment_type_suffix,
            &mut result.fragment_type_suffix,
        );
        result
    }
}

/// Visitor that prints declarations of operations and fragments.
/// Their types are provided by the type source.
pub struct OperationTypePrinterVisitor<T> {
    options: OperationTypePrinterOptions,
    types: T,
}

impl<T: OperationTypeSource> OperationTypePrinterVisitor<T> {
    pub fn new(options: OperationTypePrinterOptions, types: T) -> Self {
        Self { options, types }
    }
}

impl<T: OperationTypeSource> OperationPrinterVisitor for OperationTypePrinterVisitor<T> {
    type Error = T::Error;
    fn print_header(&self, writer: &mut impl SourceMapWriter) {
        writeln!(
            writer,
            "import type {{ TypedDocumentNode }} from \"{}\";",
            self.options.typed_document_node_source
        );
        if self.types.uses_schema_types() {
            write!(
                writer,
                "import type * as {} from \"{}\";\n\n",
                self.options.schema_root_namespace, self.options.schema_source,
            );
        } else {
            writer.write("\n");
        }
    }
    fn print_trailer(&self, _writer: &mut impl SourceMapWriter) {}
    fn print_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) -> Result<(), Self::Error> {
        let operation = &context.operation;
        let result_type_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.operation_result_type_suffix
        );
        if let Some(ref description) = operation.description {
            print_description(description, writer);
        }
        if context.export_result_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&result_type_name, &operation.name_pos());
        writer.write_for(" = ", &operation.selection_set);
        self.types
            .operation_result_type(operation)?
            .print_type(writer);
        writer.write(";\n\n");

        let input_variable_type = self.types.operation_variables_type(operation)?;
        let input_variable_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.variables_type_suffix
        );

        if context.export_input_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&input_variable_name, &operation.name_pos());
        writer.write(" = ");
        input_variable_type.print_type(writer);
        writer.write(";\n\n");

        if let Some(ref description) = operation.description {
            print_description(description, writer);
        }
        if context.exported {
            writer.write("export ");
        }
        if !self.options.print_values {
            writer.write("declare ");
        }
        writer.write("const ");
        writer.write_for(
            &context.operation_names.operation_variable_name,
            &operation.name_pos(),
        );
        writer.write_for(": ", &operation.selection_set);
        writer.write("TypedDocumentNode<");
        writer.write(&result_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        if !self.options.print_values {
            writer.write(">;\n\n");
            return Ok(());
        }
        writer.write("> = ");
        print_operation_runtime(writer, operation, context.fragments);
        // Use the `as unknown as` technique to avoid the type system complaining about
        // the type of the JSON object not matching the type of the TypedDocumentNode
        // (because of the use of enums in the TypedDocumentNode type)
        writer.write(" as unknown as TypedDocumentNode<");
        writer.write(&result_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        writer.write(">;\n\n");
        Ok(())
    }

    fn print_fragment_definition(
        &self,
        context: PrintFragmentContext,
        writer: &mut impl SourceMapWriter,
    ) -> Result<(), Self::Error> {
        let fragment = context.fragment;
        // type of fragment
        if let Some(ref description) = fragment.description {
            print_description(description, writer);
        }
        if context.exported {
            writer.write("export ");
        }
        writer.write("type ");

        let fragment_type_name = format!(
            "{}{}",
            fragment.name.name, self.options.fragment_type_suffix
        );

        writer.write_for(&fragment_type_name, fragment);

        writer.write(" = ");
        self.types.fragment_type(fragment)?.print_type(writer);
        writer.write(";\n\n");

        // runtime value
        if let Some(ref description) = fragment.description {
            print_description(description, writer);
        }
        if context.exported {
            writer.write("export ");
        }
        if !self.options.print_values {
            writer.write("declare ");
        }
        writer.write("const ");
        writer.write_for(context.var_name, fragment);
        writer.write(": ");
        writer.write("TypedDocumentNode<");
        writer.write_for(&fragment_type_name, fragment);
        writer.write(", never>");
        if !self.options.print_values {
            writer.write(";\n\n");
            return Ok(());
        }
        writer.write(" = ");
        print_fragment_runtime(writer, fragment, context.fragments);
        writer.write(" as unknown as TypedDocumentNode<");
        writer.write_for(&fragment_type_name, fragment);
        writer.write(", never>;\n\n");
        Ok(())
    }
    fn print_default_exported_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) {
        writer.write("export { ");
        writer.write(&context.operation_names.operation_variable_name);
        writer.write(" as default };\n\n");
    }
}
