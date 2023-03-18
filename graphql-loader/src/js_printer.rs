use nitrogql_ast::OperationDocument;

/// Converts given GraphQL operation file to pure JavaScript code.
/// Should match type definition printed by type_printer.
fn print_js(document: &OperationDocument) -> String {
    "".into()
}
