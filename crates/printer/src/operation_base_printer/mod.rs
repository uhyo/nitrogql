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
                    let operation_names = operation_variable_name(&self.options, def);
                    let context = PrintOperationContext {
                        operation_names: &operation_names,
                        exported: self.options.named_export_for_operation,
                        export_input_type: self.options.export_input_type,
                        export_result_type: self.options.export_result_type,
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
                        var_name: def.name.name,
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

#[derive(Debug)]
pub struct OperationNames {
    /// The (possibly capitalized) name of the operation.
    pub operation_name: String,
    /// The name of the variable that holds the operation.
    pub operation_variable_name: String,
}

/// Calculates a variable name for given operation.
pub fn operation_variable_name(
    options: &OperationBasePrinterOptions,
    operation: &OperationDefinition,
) -> OperationNames {
    let capitalized_name = if options.capitalize_operation_names {
        operation
            .name
            .map(|name| capitalize(name.name))
            .unwrap_or(String::new())
    } else {
        operation
            .name
            .map(|name| name.name.to_owned())
            .unwrap_or(String::new())
    };
    let operation_variable_name = format!(
        "{}{}",
        capitalized_name,
        match operation.operation_type {
            OperationType::Query => &options.query_variable_suffix,
            OperationType::Mutation => &options.mutation_variable_suffix,
            OperationType::Subscription => &options.subscription_variable_suffix,
        }
    );

    OperationNames {
        operation_name: capitalized_name,
        operation_variable_name,
    }
}
