use nitrogql_ast::OperationDocument;
use nitrogql_printer::{print_js_for_operation_document, OperationJSPrinterOptions};
use sourcemap_writer::SourceWriter;

pub fn print_js(document: &OperationDocument) -> String {
    let mut writer = SourceWriter::new();
    let options = OperationJSPrinterOptions::default();
    print_js_for_operation_document(options, document, &mut writer);
    let buffers = writer.into_buffers();
    buffers.buffer
}
