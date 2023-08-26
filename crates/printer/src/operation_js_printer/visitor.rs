use nitrogql_ast::{operation::ExecutableDefinition, OperationDocument};
use sourcemap_writer::SourceMapWriter;

use crate::{
    json_printer::print_to_json_string,
    operation_base_printer::{
        options::OperationBasePrinterOptions, OperationPrinterVisitor, PrintFragmentContext,
        PrintOperationContext,
    },
};

#[derive(Clone, Debug, Default)]
pub struct OperationJSPrinterOptions {
    pub base_options: OperationBasePrinterOptions,
}

pub struct OperationJSPrinterVisitor<'a, 'src> {
    context: OperationJSPrinterContext<'a, 'src>,
}

impl<'a, 'src> OperationJSPrinterVisitor<'a, 'src> {
    pub fn new(operation: &'a OperationDocument<'src>) -> Self {
        let context = OperationJSPrinterContext { operation };
        Self { context }
    }
}

pub struct OperationJSPrinterContext<'a, 'src> {
    operation: &'a OperationDocument<'src>,
}

impl<'a, 'src> OperationPrinterVisitor for OperationJSPrinterVisitor<'a, 'src> {
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
        // To follow the community conventions, generated JSON has only one operation in it
        let this_document = self
            .context
            .operation
            .definitions
            .iter()
            .filter(|def| match def {
                ExecutableDefinition::FragmentDefinition(_) => true,
                ExecutableDefinition::OperationDefinition(op) => {
                    op.name.map(|ident| ident.name) == operation.name.map(|ident| ident.name)
                }
            })
            .collect::<Vec<_>>();
        writer.write(&print_to_json_string(&this_document[..]));
        writer.write(";\n\n");
    }

    fn print_fragment_definition(
        &self,
        _context: PrintFragmentContext,
        _writer: &mut impl SourceMapWriter,
    ) {
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
