use std::{borrow::Borrow, ops::Deref};

use graphql_type_system::{DirectiveDefinition, InputValue, Schema, Text, Type, TypeDefinition};
use sourcemap_writer::SourceMapWriter;

use crate::GraphQLPrinter;

use super::utils::print_string;

impl<'a, Str: Text<'a>, OriginalNode> GraphQLPrinter for Schema<Str, OriginalNode> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        {
            let root_types = self.root_types();
            if let Some(description) = self.description() {
                print_string(&description, writer);
            }
            writer.write("schema {\n");
            writer.indent();
            if let Some(ref query_type) = root_types.query_type {
                writer.write("query: ");
                writer.write(query_type);
                writer.write("\n");
            }
            if let Some(ref mutation_type) = root_types.mutation_type {
                writer.write("mutation: ");
                writer.write(mutation_type);
                writer.write("\n");
            }
            if let Some(ref subscription_type) = root_types.subscription_type {
                writer.write("subscription: ");
                writer.write(subscription_type);
                writer.write("\n");
            }
            writer.dedent();
            writer.write("}\n");
        }
        for (_, directive) in self.iter_directives() {
            directive.print_graphql(writer);
        }
        writer.write("\n");
        for (_, def) in self.iter_types() {
            def.print_graphql(writer);
        }
    }
}

impl<'a, Str: Text<'a>, OriginalNode> GraphQLPrinter for DirectiveDefinition<Str, OriginalNode> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        print_description(&self.description, writer);
        writer.write("directive @");
        writer.write(&self.name);
        print_arguments(&self.arguments, writer);
        if self.repeatable.is_some() {
            writer.write(" repeatable")
        }
        writer.write(" on");
        for loc in self.locations.iter() {
            writer.write(" | ");
            writer.write(loc);
        }
        writer.write("\n");
    }
}

impl<'a, Str: Text<'a>, OriginalNode> GraphQLPrinter for TypeDefinition<Str, OriginalNode> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        print_description(&self.description(), writer);
        match self {
            TypeDefinition::Scalar(scalar) => {
                writer.write("scalar ");
                writer.write(&scalar.name);
                writer.write("\n\n");
            }
            TypeDefinition::Object(object) => {
                writer.write("type ");
                writer.write(&object.name);
                if !object.interfaces.is_empty() {
                    writer.write(" implements");
                    for int in object.interfaces.iter() {
                        writer.write(" & ");
                        writer.write(int);
                    }
                }
                writer.write(" {\n");
                writer.indent();
                for field in object.fields.iter() {
                    print_description(&field.description, writer);
                    writer.write(&field.name);
                    print_arguments(&field.arguments, writer);
                    writer.write(": ");
                    field.r#type.print_graphql(writer);
                    writer.write("\n");
                }
                writer.dedent();
                writer.write("}\n\n");
            }
            TypeDefinition::Interface(object) => {
                writer.write("interface ");
                writer.write(&object.name);
                if !object.interfaces.is_empty() {
                    writer.write(" implements");
                    for int in object.interfaces.iter() {
                        writer.write(" & ");
                        writer.write(int);
                    }
                }
                writer.write(" {\n");
                writer.indent();
                for field in object.fields.iter() {
                    print_description(&field.description, writer);
                    writer.write(&field.name);
                    print_arguments(&field.arguments, writer);
                    writer.write(": ");
                    field.r#type.print_graphql(writer);
                    writer.write("\n");
                }
                writer.dedent();
                writer.write("}\n\n");
            }
            TypeDefinition::Union(union) => {
                writer.write("union ");
                writer.write(&union.name);
                writer.write(" =");
                for ty in union.possible_types.iter() {
                    writer.write(" | ");
                    writer.write(ty);
                }
                writer.write("\n\n");
            }
            TypeDefinition::Enum(e) => {
                writer.write("enum ");
                writer.write(&e.name);
                writer.write(" {");
                writer.write("\n");
                writer.indent();
                for mem in e.members.iter() {
                    print_description(&mem.description, writer);
                    writer.write(&mem.name);
                    writer.write("\n");
                }
                writer.dedent();
                writer.write("}\n\n");
            }
            TypeDefinition::InputObject(object) => {
                writer.write("input ");
                writer.write(&object.name);
                writer.write(" {\n");
                writer.indent();
                for field in object.fields.iter() {
                    writer.write(&field.name);
                    writer.write(": ");
                    field.r#type.print_graphql(writer);
                    writer.write("\n");
                }
                writer.dedent();
                writer.write("}\n\n");
            }
        }
    }
}

impl<'a, Str: Text<'a>, OriginalNode> GraphQLPrinter for Type<Str, OriginalNode> {
    fn print_graphql(&self, writer: &mut impl SourceMapWriter) {
        match self {
            Type::Named(name) => {
                writer.write(name);
            }
            Type::List(ty) => {
                writer.write("[");
                ty.print_graphql(writer);
                writer.write("]");
            }
            Type::NonNull(ty) => {
                ty.print_graphql(writer);
                writer.write("!");
            }
        }
    }
}

fn print_arguments<'a, Str: Text<'a>, OriginalNode>(
    arguments: &[InputValue<Str, OriginalNode>],
    writer: &mut impl SourceMapWriter,
) {
    if arguments.is_empty() {
        return;
    }
    writer.write("(");
    let multiline = arguments.len() > 1;
    if multiline {
        writer.write("\n");
        writer.indent();
    }
    for input in arguments {
        print_description(&input.description, writer);
        writer.write(&input.name);
        writer.write(": ");
        input.r#type.print_graphql(writer);
        if let Some(ref value) = input.default_value {
            writer.write(" = ");
            writer.write(value);
        }
        if multiline {
            writer.write("\n");
        }
    }
    if multiline {
        writer.write("\n");
        writer.dedent();
    }
    writer.write(")");
}

fn print_description<Str: Borrow<str> + ?Sized, N: Deref<Target = Str>>(
    description: &Option<N>,
    writer: &mut impl SourceMapWriter,
) {
    if let Some(ref description) = description {
        print_string((**description).borrow(), writer);
        writer.write("\n");
    }
}
