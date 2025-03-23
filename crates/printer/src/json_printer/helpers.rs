//! Structs for printing GraphQL ASTs into JSON

use json_writer::{JSONObjectWriter, JSONWriter, JSONWriterValue};

use nitrogql_ast::value::Value;

use super::to_json::JsonPrintable;

pub struct JSONValue<'a, T: JsonPrintable>(pub &'a T);

impl<T> JSONWriterValue for JSONValue<'_, T>
where
    T: JsonPrintable,
{
    fn write_json<W: JSONWriter>(self, output_buffer: &mut W) {
        let mut writer = JSONObjectWriter::new(output_buffer);
        self.0.print_json(&mut writer);
        writer.end();
    }
}

pub struct Name<'a>(pub &'a str);

impl JsonPrintable for Name<'_> {
    fn print_json<W: JSONWriter>(&self, writer: &mut JSONObjectWriter<W>) {
        writer.value("kind", "Name");
        writer.value("value", self.0);
    }
}

pub struct Variable<'a> {
    name: Name<'a>,
}

impl Variable<'_> {
    pub fn new(name: Name) -> Variable {
        Variable { name }
    }
}

impl JsonPrintable for Variable<'_> {
    fn print_json<W: JSONWriter>(&self, writer: &mut JSONObjectWriter<W>) {
        writer.value("kind", "Variable");
        writer.value("name", JSONValue(&self.name));
    }
}

pub struct Argument<'a, 'b> {
    name: &'a str,
    value: &'a Value<'b>,
}

impl<'a, 'b> Argument<'a, 'b> {
    pub fn new(name: &'a str, value: &'a Value<'b>) -> Argument<'a, 'b> {
        Argument { name, value }
    }
}

impl JsonPrintable for Argument<'_, '_> {
    fn print_json<W: JSONWriter>(&self, writer: &mut JSONObjectWriter<W>) {
        writer.value("kind", "Argument");
        writer.value("name", JSONValue(&Name(self.name)));
        let mut value_writer = writer.object("value");
        self.value.print_json(&mut value_writer);
    }
}
