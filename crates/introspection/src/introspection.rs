use std::ops::Deref;

use graphql_type_system::{
    DirectiveDefinition, EnumDefinition, EnumMember, Field, InputObjectDefinition, InputValue,
    InterfaceDefinition, ListType, NamedType, Node, NonNullType, ObjectDefinition,
    ScalarDefinition, Schema, SchemaBuilder, Type, TypeDefinition, UnionDefinition,
};

use crate::{
    error::IntrospectionError,
    json_to_value::{GraphQLValue, ObjectValue},
};

/// Reads introspection json and generates schema.
pub fn introspection<D: Default>(value: &GraphQLValue<String>) -> Schema<&str, D> {
    let mut builder = SchemaBuilder::new();

    let Some(schema) = object_type_as("Schema", value) else {
        return builder.into();
    };

    if let Some(description) = schema.get("description").and_then(|v| v.as_string()) {
        builder.set_description(node(description.as_str()));
    }
    let root_types = builder.set_root_types(D::default());
    if let Some(Ok(Type::Named(query_type))) = schema.get("queryType").map(as_type::<_, ()>) {
        root_types.set_query_type(node(&query_type));
    }
    if let Some(Ok(Type::Named(mutation_type))) = schema.get("mutationType").map(as_type::<_, ()>) {
        root_types.set_mutation_type(node(&mutation_type));
    }
    if let Some(Ok(Type::Named(subscription_type))) =
        schema.get("subscriptionType").map(as_type::<_, ()>)
    {
        root_types.set_subscription_type(node(&subscription_type));
    }

    if let Some(types) = schema.get("types").and_then(|v| v.as_list()) {
        let types = types
            .values
            .iter()
            .map(as_type_definition)
            .filter_map(|v| v.ok())
            .map(node);

        builder.extend(types.map(|ty| (*ty.name(), ty)));
    }

    if let Some(directives) = schema.get("directives").and_then(|v| v.as_list()) {
        let directives = directives
            .values
            .iter()
            .map(as_directive_definition)
            .filter_map(|v| v.ok())
            .map(node);

        builder.extend(directives.map(|ty| (*ty.name(), ty)));
    }

    builder.into()
}

/// Converts given object to Type if possible.
fn as_type<'a, Str: PartialEq<&'a str> + Deref, D: Default>(
    value: &GraphQLValue<Str>,
) -> Result<Type<&Str::Target, D>, IntrospectionError> {
    let obj = object_type_as("__Type", value)
        .ok_or_else(|| IntrospectionError::Introspection("__Type expected".into()))?;
    let Some(kind) = obj.get("kind").and_then(|v| v.as_enum()) else {
        return Err(IntrospectionError::Introspection("__Type does not have the 'kind' field".into()));
    };
    if *kind == "OBJECT" {
        if let Some(name) = obj.get_str("name") {
            Ok(Type::Named(NamedType::from(node(&*name))))
        } else {
            Err(IntrospectionError::Introspection(
                "field 'name' of __Type must be a String".into(),
            ))
        }
    } else if *kind == "LIST" {
        if let Some(type_v) = obj.get("ofType") {
            let ty = as_type(type_v)?;
            Ok(Type::List(Box::new(ListType::from(ty))))
        } else {
            Err(IntrospectionError::Introspection(
                "'ofType' of __Type must exist".into(),
            ))
        }
    } else if *kind == "NON_NULL" {
        if let Some(type_v) = obj.get("ofType") {
            let ty = as_type(type_v)?;
            Ok(Type::NonNull(Box::new(NonNullType::from(ty))))
        } else {
            Err(IntrospectionError::Introspection(
                "'ofType' of __Type must exist".into(),
            ))
        }
    } else {
        Err(IntrospectionError::Introspection(
            "Invalid kind of __Type".into(),
        ))
    }
}

fn as_type_definition<'a, Str: PartialEq<&'a str> + Deref, D: Default>(
    value: &GraphQLValue<Str>,
) -> Result<TypeDefinition<&Str::Target, D>, IntrospectionError> {
    let obj = object_type_as("__Type", value)
        .ok_or_else(|| IntrospectionError::Introspection("__Type expected".into()))?;
    let Some(kind) = obj.get("kind").and_then(|v| v.as_enum()) else {
        return Err(IntrospectionError::Introspection("__Type does not have the 'kind' field".into()));
    };
    let Some(name) = obj.get("name").and_then(|v| v.as_string()).map(deref_node) else {
        return Err(IntrospectionError::Introspection("__Type must have a string 'name' field".into()));
    };
    let description = obj
        .get("description")
        .and_then(|v| v.as_string())
        .map(deref_node);

    if *kind == "SCALAR" {
        return Ok(TypeDefinition::Scalar(ScalarDefinition {
            name,
            description,
        }));
    } else if *kind == "OBJECT" {
        let Some(fields) = obj.get("fields").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind OBJECT must have a list 'fields' field".into()));
        };
        let Some(interfaces) = obj.get("interfaces").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind OBJECT must have a list 'interfaces' field".into()));
        };
        let fields = fields
            .values
            .iter()
            .map(as_field)
            .collect::<Result<Vec<_>, _>>()?;
        let interfaces = interfaces
            .values
            .iter()
            .map(as_type::<_, D>)
            .map(|ty| ty.map(|ty| ***ty.unwrapped()).map(node))
            .collect::<Result<Vec<_>, _>>()?;

        return Ok(TypeDefinition::Object(ObjectDefinition {
            name,
            description,
            fields,
            interfaces,
        }));
    } else if *kind == "INTERFACE" {
        let Some(fields) = obj.get("fields").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind INTERFACE must have a list 'fields' field".into()));
        };
        let Some(interfaces) = obj.get("interfaces").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind INTERFACE must have a list 'interfaces' field".into()));
        };
        let fields = fields
            .values
            .iter()
            .map(as_field)
            .collect::<Result<Vec<_>, _>>()?;
        let interfaces = interfaces
            .values
            .iter()
            .map(as_type::<_, D>)
            .map(|ty| ty.map(|ty| ***ty.unwrapped()).map(node))
            .collect::<Result<Vec<_>, _>>()?;

        return Ok(TypeDefinition::Interface(InterfaceDefinition {
            name,
            description,
            fields,
            interfaces,
        }));
    } else if *kind == "UNION" {
        let Some(possible_types) = obj.get("possibleTypes").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind UNION must have a list 'possibleTypes' field".into()));
        };
        let possible_types = possible_types
            .values
            .iter()
            .map(as_type::<_, D>)
            .map(|ty| ty.map(|ty| ***ty.unwrapped()).map(node))
            .collect::<Result<Vec<_>, _>>()?;

        return Ok(TypeDefinition::Union(UnionDefinition {
            name,
            description,
            possible_types,
        }));
    } else if *kind == "ENUM" {
        let Some(enum_values) = obj.get("enumValues").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind ENUM must have a list 'enumValues' field".into()));
        };
        let members = enum_values
            .values
            .iter()
            .map(|ev| {
                let ev =
                    object_type_as("__EnumValue", ev).ok_or(IntrospectionError::Introspection(
                        "Value of 'eunmValues' must be an __EnumValue".into(),
                    ))?;
                let Some(name) = ev.get("name").and_then(|v| v.as_string()).map(deref_node) else {
                    return Err(IntrospectionError::Introspection("__EnumValue must have a string 'name' field".into()));
                };
                let description = ev.get("description").and_then(|v| v.as_string()).map(deref_node);

                Ok(EnumMember {
                    name,
                    description,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        return Ok(TypeDefinition::Enum(EnumDefinition {
            name,
            description,
            members,
        }));
    } else if *kind == "INPUT_OBJECT" {
        let Some(fields) = obj.get("inputFields").and_then(|v| v.as_list()) else {
            return Err(IntrospectionError::Introspection("__Type of kind INPUT_OBJECT must have a list 'inputFields' field".into()));
        };
        let fields = fields
            .values
            .iter()
            .map(as_input_value)
            .collect::<Result<Vec<_>, _>>()?;

        return Ok(TypeDefinition::InputObject(InputObjectDefinition {
            name,
            description,
            fields,
        }));
    } else {
        Err(IntrospectionError::Introspection(
            "Unknown kind of __Type".into(),
        ))
    }
}

fn as_field<'a, Str: PartialEq<&'a str> + Deref, D: Default>(
    value: &GraphQLValue<Str>,
) -> Result<Field<&Str::Target, D>, IntrospectionError> {
    let obj = object_type_as("__Field", value)
        .ok_or_else(|| IntrospectionError::Introspection("__Field expected".into()))?;
    let Some(name) = obj.get("name").and_then(|v| v.as_string()).map(deref_node) else {
        return Err(IntrospectionError::Introspection("__Field must have a string 'name' field".into()));
    };
    let description = obj
        .get("description")
        .and_then(|v| v.as_string())
        .map(deref_node);
    let Some(ty) = obj.get("type") else {
        return Err(IntrospectionError::Introspection("'type' of __Field must be a type".into()));
    };
    let ty = as_type(ty)?;
    let arguments = obj
        .get("args")
        .and_then(|v| v.as_list())
        .map(|args| {
            args.values
                .iter()
                .map(as_input_value)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or(vec![]);

    Ok(Field {
        name,
        description,
        r#type: ty,
        arguments,
    })
}

fn as_input_value<'a, Str: PartialEq<&'a str> + Deref, D: Default>(
    value: &GraphQLValue<Str>,
) -> Result<InputValue<&Str::Target, D>, IntrospectionError> {
    let obj = object_type_as("__InputValue", value)
        .ok_or_else(|| IntrospectionError::Introspection("__InputValue expected".into()))?;
    let Some(name) = obj.get("name").and_then(|v| v.as_string()).map(deref_node) else {
        return Err(IntrospectionError::Introspection("__InputValue must have a string 'name' field".into()));
    };
    let description = obj
        .get("description")
        .and_then(|v| v.as_string())
        .map(deref_node);
    let Some(ty) = obj.get("type") else {
        return Err(IntrospectionError::Introspection("'type' of __InputValue must be a type".into()));
    };
    let ty = as_type(ty)?;
    let default_value = obj.get("default_value").and_then(|v| v.as_string());

    Ok(InputValue {
        name,
        description,
        r#type: ty,
        default_value: default_value.map(|_| node(())),
    })
}

fn as_directive_definition<'a, Str: PartialEq<&'a str> + Deref, D: Default>(
    value: &GraphQLValue<Str>,
) -> Result<DirectiveDefinition<&Str::Target, D>, IntrospectionError> {
    let obj = object_type_as("__Directive", value)
        .ok_or_else(|| IntrospectionError::Introspection("__Directive expected".into()))?;
    let Some(name) = obj.get("name").and_then(|v| v.as_string()).map(deref_node) else {
        return Err(IntrospectionError::Introspection("__Directive must have a string 'name' field".into()));
    };
    let is_repeatable = obj
        .get("isRepeatable")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);
    let description = obj
        .get("description")
        .and_then(|v| v.as_string())
        .map(deref_node);
    let Some(locations) = obj.get("locations").and_then(|v| v.as_list()) else {
        return Err(IntrospectionError::Introspection("__Directive must have a list 'locations' field".into()));
    };
    let locations = locations
        .values
        .iter()
        .map(|loc| {
            loc.as_enum()
                .ok_or(IntrospectionError::Introspection(
                    "Value of 'locations' must be an __EnumValue".into(),
                ))
                .map(deref_node)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let arguments = obj
        .get("args")
        .and_then(|v| v.as_list())
        .map(|args| {
            args.values
                .iter()
                .map(as_input_value)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or(vec![]);

    Ok(DirectiveDefinition {
        name,
        description,
        arguments,
        locations,
        repeatable: is_repeatable.then(|| node(())),
    })
}

fn node<T, D: Default>(value: T) -> Node<T, D> {
    Node::from(value, D::default())
}

fn deref_node<'a, T: ?Sized, D: Default>(value: &'a impl Deref<Target = T>) -> Node<&'a T, D> {
    node(&**value)
}

/// Returns Some if given value is an object with given `__typename`.
fn object_type_as<'a, 'b, Str: PartialEq<&'a str>>(
    expected_name: &'a str,
    value: &'b GraphQLValue<Str>,
) -> Option<&'b ObjectValue<Str>> {
    let (typename, obj) = object_typename(value)?;
    if *typename == expected_name {
        Some(obj)
    } else {
        None
    }
}

fn object_typename<'a, Str: PartialEq<&'a str>>(
    value: &GraphQLValue<Str>,
) -> Option<(&Str, &ObjectValue<Str>)> {
    match value {
        GraphQLValue::Object(obj) => {
            let (_, typename) = obj.fields.iter().find(|(key, _)| *key == "__typename")?;
            let typename = typename.as_string()?;
            Some((typename, obj))
        }
        _ => None,
    }
}
