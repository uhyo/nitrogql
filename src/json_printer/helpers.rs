//! Structs for printing GraphQL ASTs into JSON

use json_writer::{JSONObjectWriter, JSONWriterValue};

use crate::graphql_parser::ast::value::Value;

use super::to_json::JsonPrintable;

pub struct JSONValue<'a, T: JsonPrintable>(pub &'a T);

impl<'a, T> JSONWriterValue for JSONValue<'a, T>
where
    T: JsonPrintable,
{
    fn write_json(self, output_buffer: &mut String) {
        let mut writer = JSONObjectWriter::new(output_buffer);
        self.0.print_json(&mut writer);
        writer.end();
    }
}

pub struct Name<'a>(pub &'a str);

impl JsonPrintable for Name<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("type", "Name");
        writer.value("value", self.0);
    }
}

pub struct Variable<'a> {
    name: Name<'a>,
}

impl Variable<'_> {
    pub fn new<'a>(name: Name<'a>) -> Variable<'a> {
        Variable { name }
    }
}

impl JsonPrintable for Variable<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("type", "Variable");
        writer.value("variable", JSONValue(&self.name));
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
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Argument");
        writer.value("name", JSONValue(&Name(&self.name)));
        let mut value_writer = writer.object("value");
        self.value.print_json(&mut value_writer);
    }
}
