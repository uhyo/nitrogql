use std::borrow::Cow;

use graphql_type_system::Schema;
use nitrogql_ast::{OperationDocument, base::Pos};
use sourcemap_writer::SourceMapWriter;

use crate::operation_base_printer::OperationPrinter;

use self::{
    error::OperationTypePrinterResult,
    type_source::SchemaTypeSource,
    visitor::{OperationTypePrinterOptions, OperationTypePrinterVisitor},
};

pub use self::fallback::print_fallback_types_for_operation_document;

mod branching;
mod deep_merge;
pub mod error;
mod fallback;
mod selection_set_visitor;
mod selection_tree;
#[cfg(test)]
mod tests;
pub mod type_printer;
pub mod type_source;
pub mod visitor;

/// Print a TypeScript module for given operation document.
///
/// The document is assumed to be valid against the schema.
/// If it is not, an error is returned and the writer may contain partial output.
pub fn print_types_for_operation_document(
    options: OperationTypePrinterOptions,
    schema: &Schema<Cow<str>, Pos>,
    operation: &OperationDocument,
    writer: &mut impl SourceMapWriter,
) -> OperationTypePrinterResult<()> {
    let base_options = options.base_options.clone();
    let types = SchemaTypeSource::new(options.clone(), schema, operation);
    let visitor = OperationTypePrinterVisitor::new(options, types);
    let mut printer = OperationPrinter::new(base_options, visitor, writer);
    printer.print_document(operation)
}
