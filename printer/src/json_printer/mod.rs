use json_writer::JSONObjectWriter;
use to_json::JsonPrintable;

mod helpers;
mod tests;
mod to_json;

pub fn print_to_json_string<T: JsonPrintable + ?Sized>(ast: &T) -> String {
    let mut buf = String::new();
    ast.print_json(&mut JSONObjectWriter::new(&mut buf));
    buf
}
