use std::ops::Deref;

use graphql_type_system::{Node, Schema, Text};
use nitrogql_ast::{
    base::{Ident, Keyword, Pos},
    operation::OperationType,
    r#type::{ListType, NamedType, NonNullType, Type},
    type_system::{
        ArgumentsDefinition, EnumTypeDefinition, EnumValueDefinition, FieldDefinition,
        InputObjectTypeDefinition, InputValueDefinition, InterfaceTypeDefinition,
        ObjectTypeDefinition, ScalarTypeDefinition, SchemaDefinition, TypeDefinition,
        TypeSystemDefinition, UnionTypeDefinition,
    },
    value::{NullValue, StringValue, Value},
    TypeSystemDocument,
};

/// Convert Schema to TypeSystemDocument. For type definition generation purpose.
pub fn type_system_to_ast<'src, S: Text<'src>, D>(schema: &Schema<S, D>) -> TypeSystemDocument {
    let mut result = TypeSystemDocument::new();
    let schema_definition = {
        let mut schema_definition = SchemaDefinition {
            position: Pos::default(),
            description: convert_description(schema.description()),
            definitions: vec![],
            directives: vec![],
        };
        let root_types = schema.root_types();
        if let Some(ref query_type) = root_types.query_type {
            schema_definition
                .definitions
                .push((OperationType::Query, convert_node_to_ident(query_type)));
        }
        if let Some(ref mutation_type) = root_types.mutation_type {
            schema_definition.definitions.push((
                OperationType::Mutation,
                convert_node_to_ident(mutation_type),
            ));
        }
        if let Some(ref subscription_type) = root_types.subscription_type {
            schema_definition.definitions.push((
                OperationType::Subscription,
                convert_node_to_ident(subscription_type),
            ));
        }

        schema_definition
    };
    result
        .definitions
        .push(TypeSystemDefinition::SchemaDefinition(schema_definition));

    for (_, type_def) in schema.iter_types() {
        result
            .definitions
            .push(TypeSystemDefinition::TypeDefinition(
                convert_type_definition(type_def),
            ))
    }
    result
}

fn convert_type_definition<S: Deref<Target = str>, D>(
    type_def: &graphql_type_system::TypeDefinition<S, D>,
) -> TypeDefinition {
    match type_def {
        graphql_type_system::TypeDefinition::Scalar(scalar) => {
            TypeDefinition::Scalar(ScalarTypeDefinition {
                scalar_keyword: keyword("scalar"),
                position: Pos::default(),
                description: convert_description(&scalar.description),
                name: convert_node_to_ident(&scalar.name),
                directives: vec![],
            })
        }
        graphql_type_system::TypeDefinition::Object(object) => {
            TypeDefinition::Object(ObjectTypeDefinition {
                description: convert_description(&object.description),
                position: Pos::default(),
                name: convert_node_to_ident(&object.name),
                implements: object
                    .interfaces
                    .iter()
                    .map(convert_node_to_ident)
                    .collect(),
                directives: vec![],
                fields: object.fields.iter().map(convert_field).collect(),
                type_keyword: keyword("type"),
            })
        }
        graphql_type_system::TypeDefinition::Interface(interface) => {
            TypeDefinition::Interface(InterfaceTypeDefinition {
                description: convert_description(&interface.description),
                position: Pos::default(),
                name: convert_node_to_ident(&interface.name),
                directives: vec![],
                fields: interface.fields.iter().map(convert_field).collect(),
                implements: interface
                    .interfaces
                    .iter()
                    .map(convert_node_to_ident)
                    .collect(),
                interface_keyword: keyword("interface"),
            })
        }
        graphql_type_system::TypeDefinition::Union(union) => {
            TypeDefinition::Union(UnionTypeDefinition {
                description: convert_description(&union.description),
                position: Pos::default(),
                name: convert_node_to_ident(&union.name),
                directives: vec![],
                union_keyword: keyword("union"),
                members: union
                    .possible_types
                    .iter()
                    .map(convert_node_to_ident)
                    .collect(),
            })
        }
        graphql_type_system::TypeDefinition::Enum(e) => TypeDefinition::Enum(EnumTypeDefinition {
            description: convert_description(&e.description),
            position: Pos::default(),
            name: convert_node_to_ident(&e.name),
            directives: vec![],
            enum_keyword: keyword("enum"),
            values: e
                .members
                .iter()
                .map(|value| EnumValueDefinition {
                    description: convert_description(&value.description),
                    name: convert_node_to_ident(&value.name),
                    directives: vec![],
                })
                .collect(),
        }),
        graphql_type_system::TypeDefinition::InputObject(input_object) => {
            TypeDefinition::InputObject(InputObjectTypeDefinition {
                description: convert_description(&input_object.description),
                position: Pos::default(),
                name: convert_node_to_ident(&input_object.name),
                directives: vec![],
                input_keyword: keyword("input"),
                fields: input_object
                    .fields
                    .iter()
                    .map(convert_input_value)
                    .collect(),
            })
        }
    }
}

fn convert_field<S: Deref<Target = str>, D>(
    field: &graphql_type_system::Field<S, D>,
) -> FieldDefinition {
    FieldDefinition {
        description: convert_description(&field.description),
        name: convert_node_to_ident(&field.name),
        arguments: convert_arguments(&field.arguments),
        directives: vec![],
        r#type: convert_type(&field.r#type),
    }
}

fn convert_type<S: Deref<Target = str>, D>(ty: &graphql_type_system::Type<S, D>) -> Type {
    match ty {
        graphql_type_system::Type::Named(named) => Type::Named(NamedType {
            name: convert_node_to_ident(named),
        }),
        graphql_type_system::Type::List(list) => Type::List(Box::new(ListType {
            position: Pos::default(),
            r#type: convert_type(list),
        })),
        graphql_type_system::Type::NonNull(non_null) => Type::NonNull(Box::new(NonNullType {
            r#type: convert_type(non_null),
        })),
    }
}

fn convert_arguments<S: Deref<Target = str>, D>(
    arguments: &Vec<graphql_type_system::InputValue<S, D>>,
) -> Option<ArgumentsDefinition> {
    if arguments.is_empty() {
        None
    } else {
        Some(ArgumentsDefinition {
            input_values: arguments.iter().map(convert_input_value).collect(),
        })
    }
}

fn convert_input_value<S: Deref<Target = str>, D>(
    input_value: &graphql_type_system::InputValue<S, D>,
) -> InputValueDefinition {
    InputValueDefinition {
        description: convert_description(&input_value.description),
        position: Pos::default(),
        name: convert_node_to_ident(&input_value.name),
        r#type: convert_type(&input_value.r#type),
        // TODO: cannot convert default value
        default_value: input_value.default_value.as_ref().map(|_| {
            Value::NullValue(NullValue {
                position: Pos::default(),
                keyword: "null",
            })
        }),
        directives: vec![],
    }
}

fn convert_description<S: Deref<Target = str>, D>(
    description: &Option<Node<S, D>>,
) -> Option<StringValue> {
    description.as_ref().map(|desc| StringValue {
        position: Pos::default(),
        value: desc.to_string(),
    })
}

fn convert_node_to_ident<S: Deref<Target = str>, D>(node: &Node<S, D>) -> Ident {
    Ident {
        name: node,
        position: Pos::default(),
    }
}

fn keyword(name: &str) -> Keyword {
    Keyword {
        name,
        position: Pos::builtin(),
    }
}
