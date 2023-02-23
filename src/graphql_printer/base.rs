use json_writer::JSONWriterValue;

use crate::{
    graphql_parser::ast::{
        base::Ident,
        r#type::Type,
        value::{StringValue, Value},
    },
    source_map_writer::writer::SourceMapWriter,
};

use super::GraphQLPrinter;

impl GraphQLPrinter for Type<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            Type::Named(name) => {
                writer.write_for(&name.name.name, self);
            }
            Type::NonNull(non_null) => {
                non_null.r#type.print_graphql(writer);
                writer.write("!");
            }
            Type::List(list) => {
                writer.write("[");
                list.r#type.print_graphql(writer);
                writer.write("]");
            }
        }
    }
}

impl GraphQLPrinter for Value<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            Value::BooleanValue(value) => {
                writer.write(value.keyword);
            }
            Value::IntValue(value) => {
                writer.write(value.value);
            }
            Value::FloatValue(value) => {
                writer.write(value.value);
            }
            Value::StringValue(value) => {
                value.print_graphql(writer);
            }
            Value::EnumValue(value) => {
                writer.write_for(value.value, self);
            }
            Value::NullValue(value) => {
                writer.write(value.keyword);
            }
            Value::Variable(value) => {
                value.print_graphql(writer);
            }
            Value::ListValue(value) => {
                writer.write("[");
                for (idx, v) in value.values.iter().enumerate() {
                    if idx > 0 {
                        writer.write(",");
                    }
                    v.print_graphql(writer);
                }
                writer.write("]");
            }
            Value::ObjectValue(value) => {
                writer.write("{");
                if value.fields.len() < 2 {
                    for (name, value) in value.fields.iter() {
                        name.print_graphql(writer);
                        writer.write(": ");
                        value.print_graphql(writer);
                    }
                } else {
                    writer.write("\n");
                    writer.indent();
                    for (name, value) in value.fields.iter() {
                        name.print_graphql(writer);
                        writer.write(": ");
                        value.print_graphql(writer);
                        writer.write("\n");
                    }
                    writer.dedent();
                }
                writer.write("}");
            }
        }
    }
}

impl GraphQLPrinter for StringValue {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        let mut result = String::with_capacity(self.value.capacity());
        let is_multiline = self.value.find('\n').is_some();
        if is_multiline {
            // print as multiline string
            result.push_str("\"\"\"");
            let mut dq_count: usize = 0;
            for c in self.value.chars() {
                if c != '"' {
                    if dq_count > 0 {
                        result.push_str(&"\"".repeat(dq_count));
                        dq_count = 0;
                    }
                    result.push(c);
                    continue;
                }
                dq_count += 1;
                if dq_count == 3 {
                    // """ in string
                    result.push_str("\\\"\"\"");
                    dq_count = 0;
                }
            }
            if dq_count > 0 {
                result.push_str(&"\"".repeat(dq_count));
                dq_count = 0;
            }
            result.push_str("\"\"\"");
            writer.write(&result);
        } else {
            // single line string
            result.push('"');
            for c in self.value.chars() {
                match c {
                    '\r' => result.push_str("\\r"),
                    '\n' => result.push_str("\\n"),
                    c if c.is_control() => {
                        result.push_str(&format!("\\u{{{:x}}}", c as u32));
                    }
                    c => result.push(c),
                }
            }
            result.push('"');
            writer.write(&result);
        }
    }
}

impl GraphQLPrinter for Ident<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write_for(self.name, self);
    }
}
