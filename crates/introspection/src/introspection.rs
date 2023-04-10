use std::borrow::Cow;

use graphql_type_system::{
    DirectiveDefinition, EnumDefinition, EnumMember, Field, InputObjectDefinition, InputValue,
    InterfaceDefinition, ListType, NamedType, Node, NonNullType, ObjectDefinition,
    ScalarDefinition, Schema, SchemaBuilder, Type, TypeDefinition, UnionDefinition,
};
use serde::Deserialize;

use crate::error::IntrospectionError;

/// Struct that can be deserialized from results of the standard introspection query.
#[derive(Deserialize)]
pub struct IntrospectionResult<'src> {
    #[serde(rename = "__schema", borrow)]
    schema: IntrospectionSchema<'src>,
}

#[derive(Deserialize)]
struct IntrospectionSchema<'src> {
    description: Option<Cow<'src, str>>,
    #[serde(rename = "queryType", borrow)]
    query_type: NameObj<'src>,
    #[serde(rename = "mutationType")]
    mutation_type: Option<NameObj<'src>>,
    #[serde(rename = "subscriptionType")]
    subscription_type: Option<NameObj<'src>>,
    types: Vec<IntrospectionType<'src>>,
    directives: Vec<IntrospectionDirective<'src>>,
}

#[derive(Deserialize)]
struct NameObj<'src> {
    name: Cow<'src, str>,
}

#[derive(Deserialize)]
struct IntrospectionType<'src> {
    kind: Cow<'src, str>,
    name: Option<Cow<'src, str>>,
    description: Option<Cow<'src, str>>,
    fields: Option<Vec<IntrospectionField<'src>>>,
    interfaces: Option<Vec<IntrospectionType<'src>>>,
    #[serde(rename = "possibleTypes")]
    possible_types: Option<Vec<IntrospectionType<'src>>>,
    #[serde(rename = "enumValues")]
    enum_values: Option<Vec<IntrospectionEnumValue<'src>>>,
    #[serde(rename = "inputFields")]
    input_fields: Option<Vec<IntrospectionInputValue<'src>>>,
    #[serde(rename = "ofType")]
    of_type: Option<Box<IntrospectionType<'src>>>,
}

#[derive(Deserialize)]
struct IntrospectionField<'src> {
    name: Cow<'src, str>,
    description: Option<Cow<'src, str>>,
    args: Vec<IntrospectionInputValue<'src>>,
    #[serde(rename = "type")]
    ty: IntrospectionType<'src>,
    #[serde(rename = "isDeprecated")]
    #[allow(dead_code)]
    is_deprecated: bool,
    #[serde(rename = "deprecationReason")]
    #[allow(dead_code)]
    deprecation_reason: Option<Cow<'src, str>>,
}

#[derive(Deserialize)]
struct IntrospectionInputValue<'src> {
    name: Cow<'src, str>,
    description: Option<Cow<'src, str>>,
    #[serde(rename = "type")]
    ty: IntrospectionType<'src>,
    #[serde(rename = "defaultValue")]
    default_value: Option<Cow<'src, str>>,
}

#[derive(Deserialize)]
struct IntrospectionEnumValue<'src> {
    name: Cow<'src, str>,
    description: Option<Cow<'src, str>>,
    #[serde(rename = "isDeprecated")]
    #[allow(dead_code)]
    is_deprecated: bool,
    #[serde(rename = "deprecationReason")]
    #[allow(dead_code)]
    deprecation_reason: Option<Cow<'src, str>>,
}

#[derive(Deserialize)]
struct IntrospectionDirective<'src> {
    name: Cow<'src, str>,
    description: Option<Cow<'src, str>>,
    locations: Vec<Cow<'src, str>>,
    args: Vec<IntrospectionInputValue<'src>>,
    #[serde(rename = "isRepeatable")]
    is_repeatable: Option<bool>,
}

/// Reads introspection json and generates schema.
pub fn introspection<'src, D: Default>(
    value: &IntrospectionResult<'src>,
) -> Result<Schema<Cow<'src, str>, D>, IntrospectionError> {
    let mut builder = SchemaBuilder::new();

    let schema = &value.schema;

    if let Some(ref description) = schema.description {
        builder.set_description(node(description.clone()));
    }
    let root_types = builder.set_root_types(D::default());
    root_types.set_query_type(node(schema.query_type.name.clone()));
    if let Some(mutation_type) = &schema.mutation_type {
        root_types.set_mutation_type(node(mutation_type.name.clone()));
    }
    if let Some(subscription_type) = &schema.subscription_type {
        root_types.set_subscription_type(node(subscription_type.name.clone()));
    }

    let types = schema
        .types
        .iter()
        .map(as_type_definition)
        .map(|r| r.map(node))
        .collect::<Result<Vec<_>, _>>()?;

    builder.extend(types.into_iter().map(|ty| (ty.name().clone(), ty)));

    let directives = schema
        .directives
        .iter()
        .map(as_directive_definition)
        .map(|r| r.map(node))
        .collect::<Result<Vec<_>, _>>()?;

    builder.extend(directives.into_iter().map(|ty| (ty.name().clone(), ty)));

    Ok(builder.into())
}

/// Converts given object to Type if possible.
fn as_type<'src, D: Default>(
    value: &IntrospectionType<'src>,
) -> Result<Type<Cow<'src, str>, D>, IntrospectionError> {
    let kind = &value.kind;
    if matches!(
        kind.as_ref(),
        "SCALAR" | "OBJECT" | "INTERFACE" | "UNION" | "ENUM" | "INPUT_OBJECT"
    ) {
        if let Some(ref name) = value.name {
            Ok(Type::Named(NamedType::from(node_clone(name))))
        } else {
            Err(IntrospectionError::Introspection(
                "field 'name' of __Type must be a String".into(),
            ))
        }
    } else if kind == "LIST" {
        if let Some(ref type_v) = value.of_type {
            let ty = as_type(type_v)?;
            Ok(Type::List(Box::new(ListType::from(ty))))
        } else {
            Err(IntrospectionError::Introspection(
                "'ofType' of __Type must exist".into(),
            ))
        }
    } else if kind == "NON_NULL" {
        if let Some(ref type_v) = value.of_type {
            let ty = as_type(type_v)?;
            Ok(Type::NonNull(Box::new(NonNullType::from(ty))))
        } else {
            Err(IntrospectionError::Introspection(
                "'ofType' of __Type must exist".into(),
            ))
        }
    } else {
        Err(IntrospectionError::Introspection(format!(
            "Invalid kind '{kind}' of __Type"
        )))
    }
}

fn as_type_definition<'src, D: Default>(
    value: &IntrospectionType<'src>,
) -> Result<TypeDefinition<Cow<'src, str>, D>, IntrospectionError> {
    let kind = &value.kind;
    let Some(name) = value.name.as_ref().map(node_clone) else {
        return Err(IntrospectionError::Introspection(
            "field 'name' of __Type must be a String".into(),
        ));
    };
    let description = value.description.as_ref().map(node_clone);

    if kind == "SCALAR" {
        Ok(TypeDefinition::Scalar(ScalarDefinition {
            name,
            description,
        }))
    } else if kind == "OBJECT" {
        let fields = value
            .fields
            .iter()
            .flatten()
            .map(as_field)
            .collect::<Result<Vec<_>, _>>()?;
        let interfaces = value
            .interfaces
            .iter()
            .flatten()
            .map(as_type::<D>)
            .map(|ty| ty.map(|ty| (***ty.unwrapped()).clone()).map(node))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeDefinition::Object(ObjectDefinition {
            name,
            description,
            fields,
            interfaces,
        }))
    } else if kind == "INTERFACE" {
        let fields = value
            .fields
            .iter()
            .flatten()
            .map(as_field)
            .collect::<Result<Vec<_>, _>>()?;
        let interfaces = value
            .interfaces
            .iter()
            .flatten()
            .map(as_type::<D>)
            .map(|ty| ty.map(|ty| (***ty.unwrapped()).clone()).map(node))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeDefinition::Interface(InterfaceDefinition {
            name,
            description,
            fields,
            interfaces,
        }))
    } else if kind == "UNION" {
        let Some(ref possible_types) = value.possible_types else {
            return Err(IntrospectionError::Introspection("__Type of kind UNION must have a list 'possibleTypes' field".into()));
        };
        let possible_types = possible_types
            .iter()
            .map(as_type::<D>)
            .map(|ty| ty.map(|ty| (***ty.unwrapped()).clone()).map(node))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeDefinition::Union(UnionDefinition {
            name,
            description,
            possible_types,
        }))
    } else if kind == "ENUM" {
        let Some(ref enum_values) = value.enum_values else {
            return Err(IntrospectionError::Introspection("__Type of kind ENUM must have a list 'enumValues' field".into()));
        };
        let members = enum_values
            .iter()
            .map(|ev| {
                let name = node_clone(&ev.name);
                let description = ev.description.as_ref().map(node_clone);

                EnumMember { name, description }
            })
            .collect();

        Ok(TypeDefinition::Enum(EnumDefinition {
            name,
            description,
            members,
        }))
    } else if kind == "INPUT_OBJECT" {
        let Some(ref fields) = value.input_fields else {
            return Err(IntrospectionError::Introspection("__Type of kind INPUT_OBJECT must have a list 'inputFields' field".into()));
        };
        let fields = fields
            .iter()
            .map(as_input_value)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(TypeDefinition::InputObject(InputObjectDefinition {
            name,
            description,
            fields,
        }))
    } else {
        Err(IntrospectionError::Introspection(format!(
            "Unknown kind '{kind}' of __Type"
        )))
    }
}

fn as_field<'src, D: Default>(
    value: &IntrospectionField<'src>,
) -> Result<Field<Cow<'src, str>, D>, IntrospectionError> {
    let name = node_clone(&value.name);
    let description = value.description.as_ref().map(node_clone);
    let ty = as_type(&value.ty)?;
    let arguments = value
        .args
        .iter()
        .map(as_input_value)
        .collect::<Result<_, _>>()
        .unwrap_or(vec![]);

    Ok(Field {
        name,
        description,
        r#type: ty,
        arguments,
    })
}

fn as_input_value<'src, D: Default>(
    value: &IntrospectionInputValue<'src>,
) -> Result<InputValue<Cow<'src, str>, D>, IntrospectionError> {
    let name = node_clone(&value.name);
    let description = value.description.as_ref().map(node_clone);
    let ty = as_type(&value.ty)?;
    let default_value = value.default_value.as_ref().map(node_clone);

    Ok(InputValue {
        name,
        description,
        r#type: ty,
        default_value,
    })
}

fn as_directive_definition<'src, D: Default>(
    value: &IntrospectionDirective<'src>,
) -> Result<DirectiveDefinition<Cow<'src, str>, D>, IntrospectionError> {
    let name = node_clone(&value.name);
    let description = value.description.as_ref().map(node_clone);
    let locations = value.locations.iter().map(node_clone).collect();
    let arguments = value
        .args
        .iter()
        .map(as_input_value)
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or(vec![]);

    Ok(DirectiveDefinition {
        name,
        description,
        arguments,
        locations,
        repeatable: value.is_repeatable.and_then(|b| b.then(|| node(()))),
    })
}

fn node<T, D: Default>(value: T) -> Node<T, D> {
    Node::from(value, D::default())
}

fn node_clone<T: Clone, D: Default>(value: &T) -> Node<T, D> {
    Node::from(value.clone(), D::default())
}
