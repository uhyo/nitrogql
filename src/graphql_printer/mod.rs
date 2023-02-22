use json_writer::JSONWriterValue;

use crate::{
    graphql_parser::ast::{
        base::{Ident, Variable},
        directive::Directive,
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, VariableDefinition,
            VariablesDefinition,
        },
        r#type::Type,
        selection_set::{Selection, SelectionSet},
        value::{Arguments, Value},
        OperationDocument,
    },
    source_map_writer::writer::SourceMapWriter,
};

pub trait GraphQLPrinter {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter);
}

impl GraphQLPrinter for OperationDocument<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        for def in self.definitions.iter() {
            def.print_graphql(writer);
            writer.write("\n");
        }
    }
}

impl GraphQLPrinter for ExecutableDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            ExecutableDefinition::OperationDefinition(op) => {
                op.print_graphql(writer);
            }
            ExecutableDefinition::FragmentDefinition(fragment) => {
                fragment.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for OperationDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write(self.operation_type.as_str());
        if let Some(ref name) = self.name {
            writer.write(" ");
            name.print_graphql(writer);
        }
        if let Some(ref def) = self.variables_definition {
            def.print_graphql(writer);
        }
        for d in self.directives.iter() {
            writer.write(" ");
            d.print_graphql(writer);
        }
        writer.write(" ");
        self.selection_set.print_graphql(writer);
    }
}

impl GraphQLPrinter for VariablesDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("(");
        let len = self.definitions.len();
        if len < 2 {
            for d in self.definitions.iter() {
                d.print_graphql(writer);
            }
        } else {
            writer.write("\n");
            writer.indent();
            for (idx, d) in self.definitions.iter().enumerate() {
                if idx > 0 {
                    writer.write(",\n")
                }
                d.print_graphql(writer);
            }
            writer.dedent();
            writer.write("\n");
        }
        writer.write(")");
    }
}

impl GraphQLPrinter for VariableDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        self.name.print_graphql(writer);
        writer.write(": ");
        self.r#type.print_graphql(writer);
    }
}

impl GraphQLPrinter for Variable<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write_for("$", self);
        writer.write(self.name);
    }
}

impl GraphQLPrinter for Directive<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("@");
        self.name.print_graphql(writer);
        if let Some(ref arguments) = self.arguments {
            arguments.print_graphql(writer);
        }
    }
}

impl GraphQLPrinter for Arguments<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("(");
        if self.arguments.len() < 2 {
            for (name, value) in self.arguments.iter() {
                name.print_graphql(writer);
                writer.write(": ");
                value.print_graphql(writer);
            }
        } else {
            writer.write("\n");
            writer.indent();
            for (name, value) in self.arguments.iter() {
                name.print_graphql(writer);
                writer.write(": ");
                value.print_graphql(writer);
                writer.write("\n");
            }
            writer.dedent();
        }
        writer.write(")");
    }
}

impl GraphQLPrinter for SelectionSet<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("{\n");
        writer.indent();
        for selection in self.selections.iter() {
            selection.print_graphql(writer);
            writer.write("\n");
        }
        writer.dedent();
        writer.write("}");
    }
}

impl GraphQLPrinter for Selection<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            Selection::Field(field) => {
                if let Some(ref name) = field.alias {
                    name.print_graphql(writer);
                    writer.write(": ");
                }
                field.name.print_graphql(writer);
                if let Some(ref arguments) = field.arguments {
                    arguments.print_graphql(writer);
                }
                for d in field.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                if let Some(ref selection_set) = field.selection_set {
                    writer.write(" ");
                    selection_set.print_graphql(writer);
                }
            }
            Selection::FragmentSpread(spread) => {
                writer.write("... ");
                spread.fragment_name.print_graphql(writer);
                for d in spread.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
            }
            Selection::InlineFragment(fragment) => {
                writer.write("... ");
                if let Some(ref type_condition) = fragment.type_condition {
                    writer.write("on ");
                    type_condition.print_graphql(writer);
                    writer.write(" ");
                }
                for d in fragment.directives.iter() {
                    writer.write(" ");
                    d.print_graphql(writer);
                }
                fragment.selection_set.print_graphql(writer);
            }
        }
    }
}

impl GraphQLPrinter for FragmentDefinition<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write("fragment ");
        self.name.print_graphql(writer);
        writer.write(" on ");
        self.type_condition.print_graphql(writer);
        writer.write(" ");
        todo!()
    }
}

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
                let mut escaped = String::new();
                value.value.write_json(&mut escaped);
                writer.write(&escaped);
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

impl GraphQLPrinter for Ident<'_> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        writer.write_for(self.name, self);
    }
}
