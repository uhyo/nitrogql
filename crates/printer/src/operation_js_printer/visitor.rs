use nitrogql_ast::OperationDocument;
use sourcemap_writer::SourceMapWriter;

use crate::{
    json_printer::{print_to_json_string, ExecutableDefinitionRef},
    operation_base_printer::{
        OperationPrinterVisitor, PrintFragmentContext, PrintOperationContext,
    },
    utils::fragment_names_in_selection_set,
};

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
        let fragments_to_include =
            fragment_names_in_selection_set(&operation.selection_set, |name| {
                context.fragments.get(name).copied()
            })
            .into_iter()
            .map(|name| {
                ExecutableDefinitionRef::FragmentDefinition(
                    context.fragments.get(name).expect("fragment not found"),
                )
            });
        // To follow the community conventions, generated JSON has only one operation in it
        let this_document = vec![ExecutableDefinitionRef::OperationDefinition(
            context.operation,
        )]
        .into_iter()
        .chain(fragments_to_include)
        .collect::<Vec<_>>();
        writer.write(&print_to_json_string(&this_document[..]));
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

        let fragments_to_include =
            fragment_names_in_selection_set(&fragment.selection_set, |name| {
                context.fragments.get(name).copied()
            })
            .into_iter()
            .filter(|f| {
                // Filter out the fragment we are currently processing
                *f != fragment.name.name
            })
            .map(|name| {
                ExecutableDefinitionRef::FragmentDefinition(
                    context.fragments.get(name).expect("fragment not found"),
                )
            });

        // Generated document is the collection of all relevant fragments,
        // and the fragment we are currently processing
        // comes first in the list
        let this_document = vec![ExecutableDefinitionRef::FragmentDefinition(fragment)]
            .into_iter()
            .chain(fragments_to_include)
            .collect::<Vec<_>>();
        writer.write(&print_to_json_string(&this_document[..]));
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
