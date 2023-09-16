use crate::{
    ts_types::{
        ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type, ObjectField, ObjectKey,
        TSType,
    },
    utils::interface_implementers,
};
use nitrogql_ast::type_system::{
    ArgumentsDefinition, InterfaceTypeDefinition, ObjectTypeDefinition, TypeDefinition,
    UnionTypeDefinition,
};

use super::printer::ResolverTypePrinterContext;

pub fn get_ts_type_for_resolver(
    def: &TypeDefinition<'_>,
    context: &ResolverTypePrinterContext,
) -> TSType {
    let base_type = TSType::NamespaceMember(
        context.options.schema_root_namespace.clone(),
        def.name().name.to_string(),
    );
    match def {
        TypeDefinition::Object(_) => {
            // omit the __typename field for resolver logic
            TSType::TypeFunc(
                Box::new(TSType::TypeVariable("Omit".into())),
                vec![base_type, TSType::StringLiteral("__typename".to_string())],
            )
        }
        TypeDefinition::Interface(def) => {
            let implementers = interface_implementers(context.schema, def.name.name);
            ts_union(implementers.map(|obj| TSType::TypeVariable(obj.name.as_ref().into())))
        }
        TypeDefinition::Union(def) => ts_union(
            def.members
                .iter()
                .map(|type_name| TSType::TypeVariable(type_name.into())),
        ),
        _ => base_type,
    }
}

pub fn get_resolver_type(
    def: &TypeDefinition<'_>,
    context: &ResolverTypePrinterContext,
) -> Option<TSType> {
    match def {
        TypeDefinition::Scalar(_) => None,
        TypeDefinition::Object(def) => get_object_resolver_type(def, context),
        TypeDefinition::Interface(def) => get_interface_resolver_type(def, context),
        TypeDefinition::Union(def) => get_union_resolver_type(def, context),
        TypeDefinition::Enum(_) => None,
        TypeDefinition::InputObject(_) => None,
    }
}

fn get_object_resolver_type(
    def: &ObjectTypeDefinition<'_>,
    _context: &ResolverTypePrinterContext,
) -> Option<TSType> {
    let parent_type = TSType::TypeVariable((&def.name).into());
    let fields = def
        .fields
        .iter()
        .map(|field| {
            let arguments_type = field
                .arguments
                .as_ref()
                .map_or_else(|| TSType::Object(vec![]), arguments_definition_to_ts);
            let result_type = get_ts_type_of_type(&field.r#type, |name| {
                TSType::TypeVariable((&name.name).into())
            });

            let resolver_type = TSType::TypeFunc(
                Box::new(TSType::TypeVariable("__Resolver".into())),
                vec![
                    // Parent
                    parent_type.clone(),
                    // Args
                    arguments_type,
                    // Context
                    TSType::TypeVariable("Context".into()),
                    // Result
                    result_type,
                ],
            );

            ObjectField {
                key: ObjectKey::from(&field.name),
                r#type: resolver_type,
                optional: false,
                readonly: false,
                description: None,
            }
        })
        .collect();
    Some(TSType::Object(fields))
}

fn get_interface_resolver_type(
    def: &InterfaceTypeDefinition,
    context: &ResolverTypePrinterContext,
) -> Option<TSType> {
    let implementers = interface_implementers(context.schema, def.name.name);
    let (parent_types, result_types): (Vec<_>, Vec<_>) = implementers
        .map(|obj| {
            (
                TSType::TypeVariable(obj.name.as_ref().into()),
                TSType::StringLiteral(obj.name.to_string()),
            )
        })
        .unzip();

    let parent_type = ts_union(parent_types);
    let result_type = ts_union(result_types);

    let resolver_type = TSType::TypeFunc(
        Box::new(TSType::TypeVariable("__TypeResolver".into())),
        vec![
            // Parent
            parent_type,
            // Context
            TSType::TypeVariable("Context".into()),
            // Result
            result_type,
        ],
    );

    Some(TSType::object(vec![("__resolveType", resolver_type, None)]))
}

fn get_union_resolver_type(
    def: &UnionTypeDefinition,
    _context: &ResolverTypePrinterContext,
) -> Option<TSType> {
    let (parent_types, result_types): (Vec<_>, Vec<_>) = def
        .members
        .iter()
        .map(|type_name| {
            (
                TSType::TypeVariable(type_name.into()),
                TSType::StringLiteral(type_name.name.to_string()),
            )
        })
        .unzip();
    let parent_type = ts_union(parent_types);
    let result_type = ts_union(result_types);

    let resolver_type = TSType::TypeFunc(
        Box::new(TSType::TypeVariable("__TypeResolver".into())),
        vec![
            // Parent
            parent_type,
            // Context
            TSType::TypeVariable("Context".into()),
            // Result
            result_type,
        ],
    );

    Some(TSType::object(vec![("__resolveType", resolver_type, None)]))
}

fn arguments_definition_to_ts(arguments: &ArgumentsDefinition) -> TSType {
    TSType::object(arguments.input_values.iter().map(|argument| {
        (
            ObjectKey::from(&argument.name),
            get_ts_type_of_type(&argument.r#type, |name| {
                TSType::TypeVariable((&name.name).into())
            }),
            argument.description.as_ref().map(|s| s.to_string()),
        )
    }))
    .into_readonly()
}
