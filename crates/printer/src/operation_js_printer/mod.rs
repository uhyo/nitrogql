use nitrogql_ast::OperationDocument;
use sourcemap_writer::SourceMapWriter;

use crate::operation_base_printer::OperationPrinter;

use self::visitor::{OperationJSPrinterOptions, OperationJSPrinterVisitor};

pub mod visitor;

/// Print a JavaScript module for given operation document.
pub fn print_js_for_operation_document(
    options: OperationJSPrinterOptions,
    operation: &OperationDocument,
    writer: &mut impl SourceMapWriter,
) {
    let base_options = options.base_options;
    let visitor = OperationJSPrinterVisitor::new(operation);
    let mut printer = OperationPrinter::new(base_options, visitor, writer);
    printer.print_document(operation);
}
