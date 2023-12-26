use sourcemap_writer::SourceMapWriter;

use crate::operation_base_printer::{
    OperationPrinterVisitor, PrintFragmentContext, PrintOperationContext,
};

use super::printers::{print_fragment_runtime, print_operation_runtime};

pub struct OperationJSPrinterVisitor {}

impl OperationJSPrinterVisitor {
    pub fn new() -> Self {
        Self {}
    }
}

impl OperationPrinterVisitor for OperationJSPrinterVisitor {
    fn print_header(&self, _writer: &mut impl SourceMapWriter) {}
    fn print_trailer(&self, _writer: &mut impl SourceMapWriter) {}
    fn print_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) {
        let operation = &context.operation;
        if context.exported {
            writer.write("export ");
        }
        writer.write("const ");
        writer.write_for(
            &context.operation_names.operation_variable_name,
            &operation.name_pos(),
        );
        writer.write(" = ");
        print_operation_runtime(writer, operation, context.fragments);
        writer.write(";\n\n");
    }

    fn print_fragment_definition(
        &self,
        context: PrintFragmentContext,
        writer: &mut impl SourceMapWriter,
    ) {
        let fragment = context.fragment;
        // TODO: implementation is duplicated from operation_type_printer
        if context.exported {
            writer.write("export ");
        }
        writer.write("const ");

        writer.write_for(context.var_name, fragment);
        writer.write(" = ");
        print_fragment_runtime(writer, fragment, context.fragments);
        writer.write(";\n\n");
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
