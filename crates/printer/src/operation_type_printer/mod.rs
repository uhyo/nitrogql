use std::borrow::Cow;

use graphql_type_system::Schema;
use nitrogql_ast::{OperationDocument, base::Pos};
use sourcemap_writer::SourceMapWriter;

use crate::operation_base_printer::OperationPrinter;

use self::visitor::{OperationTypePrinterOptions, OperationTypePrinterVisitor};

mod branching;
mod deep_merge;
mod selection_set_visitor;
mod selection_tree;
#[cfg(test)]
mod tests;
pub mod type_printer;
pub mod visitor;

/// Print a TypeScript module for given operation document.
pub fn print_types_for_operation_document(
    options: OperationTypePrinterOptions,
    schema: &Schema<Cow<str>, Pos>,
    operation: &OperationDocument,
    writer: &mut impl SourceMapWriter,
) {
    let base_options = options.base_options.clone();
    let visitor = OperationTypePrinterVisitor::new(options, schema, operation);
    let mut printer = OperationPrinter::new(base_options, visitor, writer);
    printer.print_document(operation);
}
