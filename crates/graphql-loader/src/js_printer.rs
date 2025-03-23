use nitrogql_ast::OperationDocument;
use nitrogql_config_file::Config;
use nitrogql_printer::{OperationJSPrinterOptions, print_js_for_operation_document};
use sourcemap_writer::SourceWriter;

pub fn print_js(document: &OperationDocument, config: &Config) -> String {
    let mut writer = SourceWriter::new();
    let options = OperationJSPrinterOptions::from_config(config);
    print_js_for_operation_document(options, document, &mut writer);
    let buffers = writer.into_buffers();
    buffers.buffer
}
