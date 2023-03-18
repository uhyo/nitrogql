use nitrogql_ast::operation::{
    ExecutableDefinition, OperationDefinition, OperationDocument, OperationType,
};
use nitrogql_utils::capitalize;
use sourcemap_writer::SourceMapWriter;

use self::options::OperationBasePrinterOptions;

pub mod options;
mod visitor;

pub use visitor::{OperationPrinterVisitor, PrintFragmentContext, PrintOperationContext};

pub struct OperationPrinter<'a, Writer: SourceMapWriter, Visitor: OperationPrinterVisitor> {
    options: OperationBasePrinterOptions,
    writer: &'a mut Writer,
    visitor: Visitor,
}

impl<'a, Writer, Visitor> OperationPrinter<'a, Writer, Visitor>
where
    Writer: SourceMapWriter,
    Visitor: OperationPrinterVisitor,
{
    pub fn new(
        options: OperationBasePrinterOptions,
        visitor: Visitor,
        writer: &'a mut Writer,
    ) -> Self {
        OperationPrinter {
            options,
            writer,
            visitor,
        }
    }

    pub fn print_document(&mut self, document: &OperationDocument) {
        self.visitor.print_header(self.writer);

        let operation_count = document
            .definitions
            .iter()
            .filter(|def| matches!(def, ExecutableDefinition::OperationDefinition(_)))
            .count();

        for d in document.definitions.iter() {
            match d {
                ExecutableDefinition::OperationDefinition(ref def) => {
                    let var_name = operation_variable_name(&self.options, def);
                    let context = PrintOperationContext {
                        var_name: &var_name,
                        exported: self.options.named_export_for_operation,
                        operation: def,
                    };
                    self.visitor
                        .print_operation_definition(context, self.writer);

                    if self.options.default_export_for_operation && operation_count == 1 {
                        self.visitor
                            .print_default_exported_operation_definition(context, self.writer);
                    }
                }
                ExecutableDefinition::FragmentDefinition(ref def) => {
                    let context = PrintFragmentContext {
                        var_name: &def.name.name,
                        exported: true,
                        fragment: def,
                    };
                    self.visitor.print_fragment_definition(context, self.writer);
                }
            }
        }

        self.visitor.print_trailer(self.writer);
    }
}

/// Calculates a variable name for given operation.
pub fn operation_variable_name(
    options: &OperationBasePrinterOptions,
    operation: &OperationDefinition,
) -> String {
    let capitalized_name = operation
        .name
        .map(|name| capitalize(&name.name))
        .unwrap_or(String::new());
    format!(
        "{}{}",
        capitalized_name,
        match operation.operation_type {
            OperationType::Query => &options.query_variable_suffix,
            OperationType::Mutation => &options.mutation_variable_suffix,
            OperationType::Subscription => &options.subscription_variable_suffix,
        }
    )
}
