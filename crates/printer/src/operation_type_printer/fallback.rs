use nitrogql_ast::OperationDocument;
use sourcemap_writer::SourceMapWriter;

use crate::operation_base_printer::{
    OperationPrinter, OperationPrinterVisitor, PrintFragmentContext, PrintOperationContext,
};

use super::visitor::OperationTypePrinterOptions;

/// Print a TypeScript module for given operation document, with all types being `any`.
///
/// Unlike `print_types_for_operation_document`, this does not require the document to be valid
/// against the schema. This is useful for providing the same set of exports as usual
/// for documents that have errors.
pub fn print_fallback_types_for_operation_document(
    options: OperationTypePrinterOptions,
    operation: &OperationDocument,
    writer: &mut impl SourceMapWriter,
) {
    let base_options = options.base_options.clone();
    let visitor = FallbackOperationTypePrinterVisitor { options };
    let mut printer = OperationPrinter::new(base_options, visitor, writer);
    printer.print_document(operation);
}

struct FallbackOperationTypePrinterVisitor {
    options: OperationTypePrinterOptions,
}

impl OperationPrinterVisitor for FallbackOperationTypePrinterVisitor {
    fn print_header(&self, writer: &mut impl SourceMapWriter) {
        writeln!(
            writer,
            "import type {{ TypedDocumentNode }} from \"{}\";\n",
            self.options.typed_document_node_source
        );
    }
    fn print_trailer(&self, _writer: &mut impl SourceMapWriter) {}
    fn print_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) {
        let operation = &context.operation;
        let result_type_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.operation_result_type_suffix
        );
        let input_variable_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.variables_type_suffix
        );

        if context.export_result_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&result_type_name, &operation.name_pos());
        writer.write(" = any;\n\n");

        if context.export_input_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&input_variable_name, &operation.name_pos());
        writer.write(" = any;\n\n");

        if context.exported {
            writer.write("export ");
        }
        writer.write("declare const ");
        writer.write_for(
            &context.operation_names.operation_variable_name,
            &operation.name_pos(),
        );
        writer.write(": TypedDocumentNode<");
        writer.write(&result_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        writer.write(">;\n\n");
    }
    fn print_fragment_definition(
        &self,
        context: PrintFragmentContext,
        writer: &mut impl SourceMapWriter,
    ) {
        let fragment = context.fragment;
        let fragment_type_name = format!(
            "{}{}",
            fragment.name.name, self.options.fragment_type_suffix
        );

        if context.exported {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&fragment_type_name, fragment);
        writer.write(" = any;\n\n");

        if context.exported {
            writer.write("export ");
        }
        writer.write("declare const ");
        writer.write_for(context.var_name, fragment);
        writer.write(": TypedDocumentNode<");
        writer.write(&fragment_type_name);
        writer.write(", never>;\n\n");
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

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use nitrogql_ast::set_current_file_of_pos;
    use nitrogql_parser::parse_operation_document;
    use nitrogql_semantics::resolve_operation_extensions;
    use sourcemap_writer::JustWriter;

    use super::*;

    fn print(source: &str, options: OperationTypePrinterOptions) -> String {
        set_current_file_of_pos(0);
        let doc = parse_operation_document(source).unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        print_fallback_types_for_operation_document(options, &doc, &mut writer);
        result
    }

    #[test]
    fn fallback_types() {
        // Fields do not need to exist in any schema.
        let printed = print(
            "
            query GetMe($id: ID!) {
                me(id: $id) { unknownField }
            }
            fragment F on UnknownType {
                foo
            }
            ",
            OperationTypePrinterOptions::default(),
        );
        assert_snapshot!(printed);
    }

    #[test]
    fn fallback_types_named_exports() {
        let mut options = OperationTypePrinterOptions::default();
        options.base_options.default_export_for_operation = false;
        options.base_options.named_export_for_operation = true;
        options.base_options.export_result_type = true;
        options.base_options.export_input_type = true;
        let printed = print(
            "
            query A { a }
            mutation B { b }
            ",
            options,
        );
        assert_snapshot!(printed);
    }
}
