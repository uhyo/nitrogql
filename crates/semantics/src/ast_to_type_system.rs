use std::borrow::Cow;

use graphql_type_system::{
    DirectiveDefinition, EnumDefinition, EnumMember, Field, InputObjectDefinition, InputValue,
    InterfaceDefinition, Node, ObjectDefinition, ScalarDefinition, Schema, SchemaBuilder,
    TypeDefinition, UnionDefinition,
};
use nitrogql_ast::{
    base::{HasPos, Pos},
    directive::Directive,
    operation::OperationType,
    type_system::{
        ArgumentsDefinition, DirectiveDefinition as AstDirectiveDefinition, FieldDefinition,
        SchemaDefinition, TypeDefinition as AstTypeDefinition, TypeSystemDefinition,
    },
    value::{StringValue, Value},
    TypeSystemDocument,
};

use crate::type_system_utils::{convert_type, ident_to_node};

/// Convert TypeSystemDocument AST to type system struct.
pub fn ast_to_type_system<'src>(
    document: &TypeSystemDocument<'src>,
) -> Schema<Cow<'src, str>, Pos> {
    let mut builder = SchemaBuilder::<Cow<'src, str>, Pos>::new();
    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(def) => {
                convert_schema_definition(def, &mut builder);
            }
            TypeSystemDefinition::TypeDefinition(def) => {
                convert_type_definition(def, &mut builder);
            }
            TypeSystemDefinition::DirectiveDefinition(def) => {
                convert_directive_definition(def, &mut builder);
            }
        }
    }

    builder.into()
}

fn convert_schema_definition<'src>(
    def: &SchemaDefinition<'src>,
    builder: &mut SchemaBuilder<Cow<'src, str>, Pos>,
) {
    if let Some(ref desc) = def.description {
        builder.set_description(Node::from(desc.value.clone(), desc.position));
    }
    let root_types = builder.set_root_types(def.position);
    for (operation, def) in def.definitions.iter() {
        match operation {
            OperationType::Query => root_types.set_query_type(ident_to_node(def)),
            OperationType::Mutation => root_types.set_mutation_type(ident_to_node(def)),
            OperationType::Subscription => root_types.set_subscription_type(ident_to_node(def)),
        }
    }
}

fn convert_type_definition<'src>(
    def: &AstTypeDefinition<'src>,
    builder: &mut SchemaBuilder<Cow<'src, str>, Pos>,
) {
    match def {
        AstTypeDefinition::Scalar(def) => builder
            .extend::<Vec<(_, Node<TypeDefinition<_, _>, _>)>>(vec![(
                def.name.name.into(),
                Node::from(
                    TypeDefinition::Scalar(ScalarDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                    }),
                    def.position,
                ),
            )]),
        AstTypeDefinition::Object(def) => builder
            .extend::<Vec<(_, Node<TypeDefinition<_, _>, _>)>>(vec![(
                def.name.name.into(),
                Node::from(
                    TypeDefinition::Object(ObjectDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        fields: def.fields.iter().map(convert_field).collect(),
                        interfaces: def.implements.iter().map(ident_to_node).collect(),
                    }),
                    def.position,
                ),
            )]),
        AstTypeDefinition::Interface(def) => builder
            .extend::<Vec<(_, Node<TypeDefinition<_, _>, _>)>>(vec![(
                def.name.name.into(),
                Node::from(
                    TypeDefinition::Interface(InterfaceDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        fields: def.fields.iter().map(convert_field).collect(),
                        interfaces: def.implements.iter().map(ident_to_node).collect(),
                    }),
                    def.position,
                ),
            )]),
        AstTypeDefinition::Union(def) => builder
            .extend::<Vec<(_, Node<TypeDefinition<_, _>, _>)>>(vec![(
                def.name.name.into(),
                Node::from(
                    TypeDefinition::Union(UnionDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        possible_types: def.members.iter().map(ident_to_node).collect(),
                    }),
                    def.position,
                ),
            )]),
        AstTypeDefinition::Enum(def) => builder
            .extend::<Vec<(_, Node<TypeDefinition<_, _>, _>)>>(vec![(
                def.name.name.into(),
                Node::from(
                    TypeDefinition::Enum(EnumDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        members: def
                            .values
                            .iter()
                            .map(|mem| EnumMember {
                                name: ident_to_node(&mem.name),
                                description: convert_description(&mem.description),
                                deprecation: convert_deprecation(&mem.directives),
                            })
                            .collect(),
                    }),
                    def.position,
                ),
            )]),
        AstTypeDefinition::InputObject(def) => {
            builder.extend::<Vec<(_, Node<TypeDefinition<_, _>, _>)>>(vec![(
                def.name.name.into(),
                Node::from(
                    TypeDefinition::InputObject(InputObjectDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        fields: def
                            .fields
                            .iter()
                            .map(|input| InputValue {
                                name: ident_to_node(&input.name),
                                description: convert_description(&input.description),
                                r#type: convert_type(&input.r#type),
                                default_value: input.default_value.as_ref().map(|value| {
                                    // TODO: do not leak
                                    let value_disp = value.to_string().into_boxed_str();
                                    let value_disp = Box::leak(value_disp);
                                    Node::from(&*value_disp, *value.position())
                                }),
                                deprecation: convert_deprecation(&input.directives),
                            })
                            .collect(),
                    }),
                    def.position,
                ),
            )])
        }
    }
}

fn convert_directive_definition<'str>(
    def: &AstDirectiveDefinition<'str>,
    builder: &mut SchemaBuilder<Cow<'str, str>, Pos>,
) {
    builder.extend::<Vec<(_, Node<DirectiveDefinition<_, _>, _>)>>(vec![(
        def.name.name.into(),
        Node::from(
            DirectiveDefinition {
                name: ident_to_node(&def.name),
                description: convert_description(&def.description),
                locations: def
                    .locations
                    .iter()
                    .map(|loc| Node::from(loc.name, loc.position))
                    .collect(),
                arguments: convert_arguments(&def.arguments),
                repeatable: def.repeatable.map(|ident| Node::from((), ident.position)),
            },
            def.position,
        ),
    )])
}

fn convert_description(description: &Option<StringValue>) -> Option<Node<Cow<'static, str>, Pos>> {
    description
        .as_ref()
        .map(|desc| Node::from(desc.value.clone(), desc.position))
}

fn convert_arguments<'src>(
    arguments: &Option<ArgumentsDefinition<'src>>,
) -> Vec<InputValue<Cow<'src, str>, Pos>> {
    arguments.as_ref().map_or(vec![], |args| {
        args.input_values
            .iter()
            .map(|input| InputValue {
                name: ident_to_node(&input.name),
                description: convert_description(&input.description),
                r#type: convert_type(&input.r#type),
                default_value: input.default_value.as_ref().map(|value| {
                    // TODO: do not leak
                    let value_disp = value.to_string().into_boxed_str();
                    let value_disp = Box::leak(value_disp);
                    Node::from(&*value_disp, *value.position())
                }),
                deprecation: convert_deprecation(&input.directives),
            })
            .collect()
    })
}

fn convert_field<'src>(field: &FieldDefinition<'src>) -> Field<Cow<'src, str>, Pos> {
    Field {
        name: ident_to_node(&field.name),
        description: convert_description(&field.description),
        r#type: convert_type(&field.r#type),
        arguments: convert_arguments(&field.arguments),
        deprecation: convert_deprecation(&field.directives),
    }
}

fn convert_deprecation<'src>(directives: &[Directive<'src>]) -> Option<Cow<'src, str>> {
    directives
        .iter()
        .find(|dir| dir.name.name == "deprecated")
        .map(|dir| {
            dir.arguments
                .iter()
                .flat_map(|args| args.arguments.iter())
                .find(|(name, _)| name.name == "reason")
                .map(|(_, value)| value)
                .and_then(|value| match value {
                    Value::StringValue(string) => Some(Cow::Owned(string.value.clone())),
                    _ => None,
                })
                .unwrap_or(
                    // Default value is from the spec
                    Cow::Borrowed("No longer supported"),
                )
        })
}
