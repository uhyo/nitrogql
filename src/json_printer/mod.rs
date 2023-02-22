use json_writer::JSONObjectWriter;
use to_json::JsonPrintable;

use crate::graphql_parser::ast::operations::ExecutableDefinition;

mod helpers;
mod tests;
pub mod to_json;

pub fn print_executable_document(ast: &ExecutableDefinition, buf: &mut String) {
    ast.print_json(&mut JSONObjectWriter::new(buf));
}
