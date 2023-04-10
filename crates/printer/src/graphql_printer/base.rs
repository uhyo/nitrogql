use nitrogql_ast::{
    base::Ident,
    r#type::Type,
    value::{StringValue, Value},
};
use sourcemap_writer::SourceMapWriter;

use super::utils::print_string;
use super::GraphQLPrinter;

impl GraphQLPrinter for Type<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            Type::Named(name) => {
                writer.write_for(name.name.name, self);
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
        print_string(&self.value, writer);
    }
}

impl GraphQLPrinter for Ident<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write_for(self.name, self);
    }
}
