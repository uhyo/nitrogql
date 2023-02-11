use graphql_parser::query::Document;
use json_writer::JSONObjectWriter;
use to_json::JsonPrintable;

mod helpers;
pub mod to_json;

pub fn print_query(ast: &Document<String>, buf: &mut String) {
    ast.print_json(&mut JSONObjectWriter::new(buf));
}
