use graphql_type_system::{
    DirectiveDefinition, EnumDefinition, EnumMember, Field, InputObjectDefinition, InputValue,
    InterfaceDefinition, ListType, NamedType, Node, NonNullType, ObjectDefinition,
    ScalarDefinition, Schema, SchemaBuilder, Type, TypeDefinition, UnionDefinition,
};
use nitrogql_ast::{
    base::{HasPos, Ident, Pos},
    operation::OperationType,
    r#type::Type as AstType,
    type_system::{
        ArgumentsDefinition, FieldDefinition, TypeDefinition as AstTypeDefinition,
        TypeSystemDefinition,
    },
    value::StringValue,
    TypeSystemDocument,
};

/// Convert TypeSystemDocument AST to type system struct.
pub fn ast_to_type_system<'a>(document: &'a TypeSystemDocument) -> Schema<&'a str, Pos> {
    let mut builder = SchemaBuilder::<&'a str, Pos>::new();
    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(ref def) => {
                if let Some(ref desc) = def.description {
                    builder.set_description(Node::from(&desc.value, desc.position));
                }
                for (operation, def) in def.definitions.iter() {
                    match operation {
                        OperationType::Query => builder.set_root_query_type(ident_to_node(def)),
                        OperationType::Mutation => {
                            builder.set_root_mutation_type(ident_to_node(def))
                        }
                        OperationType::Subscription => {
                            builder.set_root_subscription_type(ident_to_node(def))
                        }
                    }
                }
            }
            TypeSystemDefinition::TypeDefinition(ref def) => match def {
                AstTypeDefinition::Scalar(ref def) => builder.extend(vec![(
                    def.name.name,
                    TypeDefinition::Scalar(ScalarDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                    }),
                )]),
                AstTypeDefinition::Object(ref def) => builder.extend(vec![(
                    def.name.name,
                    TypeDefinition::Object(ObjectDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        fields: def.fields.iter().map(convert_field).collect(),
                        interfaces: def.implements.iter().map(ident_to_node).collect(),
                    }),
                )]),
                AstTypeDefinition::Interface(ref def) => builder.extend(vec![(
                    def.name.name,
                    TypeDefinition::Interface(InterfaceDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        fields: def.fields.iter().map(convert_field).collect(),
                        interfaces: def.implements.iter().map(ident_to_node).collect(),
                    }),
                )]),
                AstTypeDefinition::Union(ref def) => builder.extend(vec![(
                    def.name.name,
                    TypeDefinition::Union(UnionDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        possible_types: def.members.iter().map(ident_to_node).collect(),
                    }),
                )]),
                AstTypeDefinition::Enum(ref def) => builder.extend(vec![(
                    def.name.name,
                    TypeDefinition::Enum(EnumDefinition {
                        name: ident_to_node(&def.name),
                        description: convert_description(&def.description),
                        members: def
                            .values
                            .iter()
                            .map(|mem| EnumMember {
                                name: ident_to_node(&mem.name),
                            })
                            .collect(),
                    }),
                )]),
                AstTypeDefinition::InputObject(ref def) => builder.extend(vec![(
                    def.name.name,
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
                                default_value: input
                                    .default_value
                                    .as_ref()
                                    .map(|value| Node::from((), *value.position())),
                            })
                            .collect(),
                    }),
                )]),
            },
            TypeSystemDefinition::DirectiveDefinition(ref def) => builder.extend(vec![(
                def.name.name,
                DirectiveDefinition {
                    name: ident_to_node(&def.name),
                    description: convert_description(&def.description),
                    locations: def
                        .locations
                        .iter()
                        .map(|loc| Node::from(loc.name, loc.position))
                        .collect(),
                    arguments: convert_arguments(&def.arguments),
                },
            )]),
        }
    }

    builder.into()
}

fn ident_to_node<'src>(ident: &Ident<'src>) -> Node<&'src str, Pos> {
    Node::from(ident.name, ident.position)
}

fn convert_description<'src>(description: &Option<StringValue>) -> Option<Node<&str, Pos>> {
    description
        .as_ref()
        .map(|desc| Node::from(desc.value.as_ref(), desc.position))
}

fn convert_type<'src>(ty: &AstType<'src>) -> Type<&'src str, Pos> {
    match ty {
        AstType::Named(ty) => Type::Named(NamedType::from(ident_to_node(&ty.name))),
        AstType::List(ty) => Type::List(Box::new(ListType::from(convert_type(&ty.r#type)))),
        AstType::NonNull(ty) => {
            Type::NonNull(Box::new(NonNullType::from(convert_type(&ty.r#type))))
        }
    }
}

fn convert_arguments<'src, 'a: 'src>(
    arguments: &'a Option<ArgumentsDefinition<'src>>,
) -> Vec<InputValue<&'src str, Pos>> {
    arguments.as_ref().map_or(vec![], |args| {
        args.input_values
            .iter()
            .map(|input| InputValue {
                name: ident_to_node(&input.name),
                description: convert_description(&input.description),
                r#type: convert_type(&input.r#type),
                default_value: input
                    .default_value
                    .as_ref()
                    .map(|value| Node::from((), *value.position())),
            })
            .collect()
    })
}

fn convert_field<'src, 'a: 'src>(field: &'a FieldDefinition<'src>) -> Field<&'src str, Pos> {
    Field {
        name: ident_to_node(&field.name),
        description: convert_description(&field.description),
        r#type: convert_type(&field.r#type),
        arguments: convert_arguments(&field.arguments),
    }
}
