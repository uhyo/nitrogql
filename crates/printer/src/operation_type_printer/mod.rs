use std::borrow::Cow;

use graphql_type_system::Schema;
use nitrogql_ast::{OperationDocument, base::Pos};
use sourcemap_writer::SourceMapWriter;

use crate::operation_base_printer::OperationPrinter;

use self::{
    error::OperationTypePrinterResult,
    type_source::{OperationTypeSource, SchemaTypeSource},
    visitor::{OperationTypePrinterOptions, OperationTypePrinterVisitor},
};

mod branching;
mod deep_merge;
pub mod error;
pub mod fragment_imports;
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
    let types = SchemaTypeSource::new(&options, schema, operation);
    print_types_with_source(&options, types, operation, writer)?;
    Ok(())
}

/// Print a TypeScript module for given operation document, with types from given type source.
///
/// Returns the type source, so that the caller can take results collected in it.
/// If the type source fails, the error is returned and the writer may contain partial output.
pub fn print_types_with_source<T: OperationTypeSource>(
    options: &OperationTypePrinterOptions,
    types: T,
    operation: &OperationDocument,
    writer: &mut impl SourceMapWriter,
) -> Result<T, T::Error> {
    let visitor = OperationTypePrinterVisitor::new(options, types);
    let mut printer = OperationPrinter::new(options.base_options.clone(), visitor, writer);
    printer.print_document(operation)?;
    Ok(printer.into_visitor().into_types())
}
