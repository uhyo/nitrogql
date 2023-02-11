use graphql_parser::{
    query::{
        Definition, Document, Field, FragmentSpread, InlineFragment, OperationDefinition,
        Selection, SelectionSet, TypeCondition, VariableDefinition,
    },
    schema::{Directive, Text, Type, Value},
};
use json_writer::JSONObjectWriter;

use super::helpers::{Argument, JSONValue, Name, Variable};

/// Value that can be printed into JSON.
pub trait JsonPrintable {
    fn print_json(&self, writer: &mut JSONObjectWriter);
}

impl JsonPrintable for Document<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Document");

        let mut definitions_writer = writer.array("definitions");
        for d in &self.definitions {
            match d {
                Definition::Operation(op) => {
                    op.print_json(&mut definitions_writer.object());
                }
                _ => {
                    todo!("operation definition")
                }
            }
        }
    }
}

impl JsonPrintable for OperationDefinition<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "OperationDefinition");
        match self {
            OperationDefinition::Query(query) => {
                writer.value("operation", "query");
                if let Some(name) = &query.name {
                    writer.value("name", JSONValue(&Name(name.as_str())));
                }
                let mut variable_definitions_writer = writer.array("variableDefinitions");
                for v in &query.variable_definitions {
                    v.print_json(&mut variable_definitions_writer.object());
                }
                variable_definitions_writer.end();
                let mut directives_writer = writer.array("directives");
                for v in &query.directives {
                    v.print_json(&mut directives_writer.object());
                }
                directives_writer.end();
                write_selection_set(&query.selection_set, writer);
            }
            _ => {}
        }
    }
}

impl JsonPrintable for VariableDefinition<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "VariableDefinition");
        writer.value(
            "variable",
            JSONValue(&Variable::new(Name(self.name.as_str()))),
        );
        writer.value("type", JSONValue(&self.var_type));
        todo!("More fields to add")
    }
}

impl JsonPrintable for Directive<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Directive");
        writer.value("name", JSONValue(&Name(self.name.as_str())));
        let mut arguments_writer = writer.array("arguments");
        for (name, value) in &self.arguments {
            Argument::new(name, value).print_json(&mut arguments_writer.object());
        }
    }
}

impl<'a, T> JsonPrintable for Type<'a, T>
where
    T: Text<'a>,
{
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        match self {
            Type::NonNullType(inner) => {
                writer.value("kind", "NonNullType");
                inner.print_json(&mut writer.object("type"));
            }
            Type::NamedType(inner) => {
                writer.value("kind", "NamedType");
                writer.value("name", JSONValue(&Name(inner.as_ref())));
            }
            Type::ListType(inner) => {
                writer.value("kind", "ListType");
                inner.print_json(&mut writer.object("type"));
            }
        }
    }
}

impl JsonPrintable for Value<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        match self {
            Value::Variable(v) => {
                Variable::new(Name(v)).print_json(writer);
            }
            Value::Boolean(b) => {
                writer.value("kind", "BooleanValue");
                writer.value("value", *b);
            }
            Value::Int(i) => {
                writer.value("kind", "IntValue");
                writer.value("value", &i.as_i64().unwrap().to_string());
            }
            Value::Float(f) => {
                writer.value("kind", "FloatValue");
                writer.value("value", &f.to_string());
            }
            Value::String(s) => {
                writer.value("kind", "StringValue");
                writer.value("value", s);
            }
            Value::Null => {
                writer.value("kind", "NulLValue");
            }
            Value::List(list) => {
                writer.value("kind", "ListValue");
                let mut values_writer = writer.array("values");
                for v in list {
                    v.print_json(&mut values_writer.object());
                }
            }
            Value::Object(obj) => {
                writer.value("kind", "ObjectValue");
                let mut fields_writer = writer.array("fields");
                for (key, value) in obj {
                    let mut field_writer = fields_writer.object();
                    field_writer.value("kind", "ObjectField");
                    field_writer.value("name", JSONValue(&Name(key)));
                    value.print_json(&mut field_writer.object("value"));
                }
            }
            Value::Enum(e) => {
                writer.value("kind", "EnumValue");
                writer.value("value", e);
            }
        }
    }
}

impl JsonPrintable for SelectionSet<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "SelectionSet");
        let mut selections_writer = writer.array("selections");
        for selection in self.items.iter() {
            selection.print_json(&mut selections_writer.object());
        }
    }
}

impl JsonPrintable for Selection<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        match self {
            Selection::Field(field) => {
                field.print_json(writer);
            }
            Selection::FragmentSpread(frag) => {
                frag.print_json(writer);
            }
            Selection::InlineFragment(frag) => {
                frag.print_json(writer);
            }
        }
    }
}

impl JsonPrintable for Field<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Field");
        writer.value("name", JSONValue(&Name(&self.name)));
        if let Some(ref alias) = self.alias {
            writer.value("alias", JSONValue(&Name(alias)));
        }

        let mut arguments_writer = writer.array("arguments");
        for (name, value) in self.arguments.iter() {
            Argument::new(name, value).print_json(&mut arguments_writer.object());
        }
        arguments_writer.end();
        let mut directives_writer = writer.array("directives");
        for d in self.directives.iter() {
            d.print_json(&mut directives_writer.object());
        }
        directives_writer.end();
        write_selection_set(&self.selection_set, writer);
    }
}

impl JsonPrintable for FragmentSpread<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "FragmentSpread");
        writer.value("name", JSONValue(&Name(&self.fragment_name)));
        let mut directives_writer = writer.array("directives");
        for d in self.directives.iter() {
            d.print_json(&mut directives_writer.object());
        }
    }
}

impl JsonPrintable for InlineFragment<'_, String> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "InlineFragment");
        if let Some(ref cond) = self.type_condition {
            match cond {
                TypeCondition::On(ref ty) => {
                    let mut type_condition_writer = writer.object("typeCondition");
                    Type::NamedType::<&str>(ty).print_json(&mut type_condition_writer);
                }
            }
        }
        let mut directives_writer = writer.array("directives");
        for d in self.directives.iter() {
            d.print_json(&mut directives_writer.object());
        }
        directives_writer.end();
        write_selection_set(&self.selection_set, writer);
    }
}

fn write_selection_set(set: &SelectionSet<'_, String>, writer: &mut JSONObjectWriter) {
    if set.items.is_empty() {
        return;
    }
    let mut selection_set_writer = writer.object("selectionSet");
    set.print_json(&mut selection_set_writer);
}
