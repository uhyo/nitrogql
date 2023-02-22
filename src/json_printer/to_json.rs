use json_writer::JSONObjectWriter;

use crate::graphql_parser::ast::{
    directive::Directive,
    operations::{ExecutableDefinition, VariableDefinition},
    r#type::{NamedType, Type},
    selection_set::{Field, FragmentSpread, InlineFragment, Selection, SelectionSet},
    value::Value,
    OperationDocument,
};

use super::helpers::{Argument, JSONValue, Name, Variable};

/// Value that can be printed into JSON.
pub trait JsonPrintable {
    fn print_json(&self, writer: &mut JSONObjectWriter);
}

impl JsonPrintable for OperationDocument<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Document");

        let mut definitions_writer = writer.array("definitions");
        for d in &self.definitions {
            d.print_json(&mut definitions_writer.object());
        }
    }
}

impl JsonPrintable for ExecutableDefinition<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "OperationDefinition");
        match self {
            ExecutableDefinition::OperationDefinition(op) => {
                writer.value("operation", op.operation_type.as_str());
                if let Some(name) = &op.name {
                    writer.value("name", JSONValue(&Name(name.name)));
                }
                let mut variable_definitions_writer = writer.array("variableDefinitions");
                if let Some(ref def) = op.variables_definition {
                    for v in def.definitions.iter() {
                        v.print_json(&mut variable_definitions_writer.object());
                    }
                }
                variable_definitions_writer.end();
                let mut directives_writer = writer.array("directives");
                for v in &op.directives {
                    v.print_json(&mut directives_writer.object());
                }
                directives_writer.end();
                write_selection_set(&op.selection_set, writer);
            }
            ExecutableDefinition::FragmentDefinition(fragment) => {
                writer.value("kind", "FragmentDefinition");
                writer.value("name", JSONValue(&Name(&fragment.name.name)));
                Type::Named(NamedType {
                    name: fragment.type_condition.clone(),
                })
                .print_json(&mut writer.object("typeCondition"));
                let mut directives_writer = writer.array("directives");
                for d in fragment.directives.iter() {
                    d.print_json(&mut directives_writer.object());
                }
                directives_writer.end();
                write_selection_set(&fragment.selection_set, writer);
            }
        }
    }
}

impl JsonPrintable for VariableDefinition<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "VariableDefinition");
        writer.value("variable", JSONValue(&Variable::new(Name(self.name.name))));
        writer.value("type", JSONValue(&self.r#type));
        if let Some(ref value) = self.default_value {
            value.print_json(&mut writer.object("defaultValue"));
        }
        writer.array("directives").end();
    }
}

impl JsonPrintable for Directive<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Directive");
        writer.value("name", JSONValue(&Name(self.name.name)));
        let mut arguments_writer = writer.array("arguments");
        if let Some(ref arguments) = self.arguments {
            for (name, value) in arguments.arguments.iter() {
                Argument::new(name.name, value).print_json(&mut arguments_writer.object());
            }
        }
    }
}

impl JsonPrintable for Type<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        match self {
            Type::NonNull(inner) => {
                writer.value("kind", "NonNullType");
                inner.r#type.print_json(&mut writer.object("type"));
            }
            Type::Named(inner) => {
                writer.value("kind", "NamedType");
                writer.value("name", JSONValue(&Name(inner.name.name)));
            }
            Type::List(inner) => {
                writer.value("kind", "ListType");
                inner.r#type.print_json(&mut writer.object("type"));
            }
        }
    }
}

impl JsonPrintable for Value<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        match self {
            Value::Variable(v) => {
                Variable::new(Name(v.name)).print_json(writer);
            }
            Value::BooleanValue(b) => {
                writer.value("kind", "BooleanValue");
                writer.value("value", b.value);
            }
            Value::IntValue(i) => {
                writer.value("kind", "IntValue");
                writer.value("value", i.value);
            }
            Value::FloatValue(f) => {
                writer.value("kind", "FloatValue");
                writer.value("value", &f.value);
            }
            Value::StringValue(s) => {
                writer.value("kind", "StringValue");
                writer.value("value", s.value);
            }
            Value::NullValue(_) => {
                writer.value("kind", "NullValue");
            }
            Value::ListValue(list) => {
                writer.value("kind", "ListValue");
                let mut values_writer = writer.array("values");
                for v in list.values.iter() {
                    v.print_json(&mut values_writer.object());
                }
            }
            Value::ObjectValue(obj) => {
                writer.value("kind", "ObjectValue");
                let mut fields_writer = writer.array("fields");
                for (key, value) in obj.fields.iter() {
                    let mut field_writer = fields_writer.object();
                    field_writer.value("kind", "ObjectField");
                    field_writer.value("name", JSONValue(&Name(&key.name)));
                    value.print_json(&mut field_writer.object("value"));
                }
            }
            Value::EnumValue(e) => {
                writer.value("kind", "EnumValue");
                writer.value("value", e.value);
            }
        }
    }
}

impl JsonPrintable for SelectionSet<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "SelectionSet");
        let mut selections_writer = writer.array("selections");
        for selection in self.selections.iter() {
            selection.print_json(&mut selections_writer.object());
        }
    }
}

impl JsonPrintable for Selection<'_> {
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

impl JsonPrintable for Field<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "Field");
        writer.value("name", JSONValue(&Name(self.name.name)));
        if let Some(ref alias) = self.alias {
            writer.value("alias", JSONValue(&Name(alias.name)));
        }

        let mut arguments_writer = writer.array(".arguments");
        if let Some(ref arguments) = self.arguments {
            for (name, value) in arguments.arguments.iter() {
                Argument::new(name.name, value).print_json(&mut arguments_writer.object());
            }
        }
        arguments_writer.end();
        let mut directives_writer = writer.array("directives");
        for d in self.directives.iter() {
            d.print_json(&mut directives_writer.object());
        }
        directives_writer.end();
        if let Some(ref selection_set) = self.selection_set {
            write_selection_set(selection_set, writer);
        }
    }
}

impl JsonPrintable for FragmentSpread<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "FragmentSpread");
        writer.value("name", JSONValue(&Name(&self.fragment_name.name)));
        let mut directives_writer = writer.array("directives");
        for d in self.directives.iter() {
            d.print_json(&mut directives_writer.object());
        }
    }
}

impl JsonPrintable for InlineFragment<'_> {
    fn print_json(&self, writer: &mut JSONObjectWriter) {
        writer.value("kind", "InlineFragment");
        if let Some(ref cond) = self.type_condition {
            let mut type_condition_writer = writer.object("typeCondition");
            Type::Named(NamedType { name: cond.clone() }).print_json(&mut type_condition_writer);
        }
        let mut directives_writer = writer.array("directives");
        for d in self.directives.iter() {
            d.print_json(&mut directives_writer.object());
        }
        directives_writer.end();
        write_selection_set(&self.selection_set, writer);
    }
}

fn write_selection_set(set: &SelectionSet<'_>, writer: &mut JSONObjectWriter) {
    if set.selections.is_empty() {
        return;
    }
    let mut selection_set_writer = writer.object("selectionSet");
    set.print_json(&mut selection_set_writer);
}
