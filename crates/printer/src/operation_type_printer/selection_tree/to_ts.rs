use graphql_type_system::{NamedType, Text, Type};
use nitrogql_config_file::TypeTarget;

use crate::ts_types::{ts_types_util::ts_union, ObjectField, TSType};

use super::{SelectionTree, SelectionTreeField};

#[derive(Debug, Copy, Clone)]
pub struct GenerateSelectionTreeTypeContext<'a> {
    schema_root_namespace: &'a str,
}

/// Generate a TypeScript representation of the selection tree.
pub fn generate_selection_tree_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    tree: &SelectionTree<S>,
) -> TSType {
    let selection_set_utility = TSType::NamespaceMember(
        context.schema_root_namespace.into(),
        "__SelectionSet".into(),
    );
    let schema_type = TSType::NamespaceMember3(
        context.schema_root_namespace.into(),
        TypeTarget::OperationOutput.to_string(),
        tree.type_name.clone(),
    );
    let unaliased_object = TSType::Object(
        tree.unaliased_fields
            .iter()
            .map(|field| field_to_type(context, field))
            .collect(),
    );
    let aliased_object = TSType::Object(
        tree.aliased_fields
            .iter()
            .map(|field| field_to_type(context, field))
            .collect(),
    );
    TSType::TypeFunc(
        Box::new(selection_set_utility),
        vec![schema_type, unaliased_object, aliased_object],
    )
}

fn field_to_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    field: &SelectionTreeField<S>,
) -> ObjectField {
    match field {
        SelectionTreeField::Leaf(leaf) => {
            let field_type = if leaf.name == "__typename" {
                // special case for __typename
                TSType::StringLiteral(leaf.name.to_string())
            } else {
                map_to_tstype(&leaf.r#type, |ty| {
                    TSType::NamespaceMember3(
                        context.schema_root_namespace.into(),
                        TypeTarget::OperationOutput.to_string(),
                        ty.to_string(),
                    )
                })
            };
            ObjectField {
                key: leaf.name.to_string().into(),
                r#type: field_type,
                description: None,
                optional: false,
                readonly: true,
            }
        }
        SelectionTreeField::Object(object) => ObjectField {
            key: object.name.clone().into(),
            r#type: generate_selection_tree_type(context, &object.selection),
            description: None,
            optional: false,
            readonly: true,
        },
        SelectionTreeField::Branch(branch) => ObjectField {
            key: branch.name.clone().into(),
            r#type: ts_union(
                branch
                    .selections
                    .iter()
                    .map(|selection| generate_selection_tree_type(context, selection)),
            ),
            description: None,
            optional: false,
            readonly: true,
        },
    }
}

/// Map given Type to TSType.
fn map_to_tstype<Str, OriginalNode>(
    ty: &Type<Str, OriginalNode>,
    mapper: impl FnOnce(&NamedType<Str, OriginalNode>) -> TSType,
) -> TSType {
    let (res, nullable) = map_to_tstype_impl(ty, mapper);
    if nullable {
        ts_union(vec![res, TSType::Null])
    } else {
        res
    }
}

fn map_to_tstype_impl<Str, OriginalNode>(
    ty: &Type<Str, OriginalNode>,
    mapper: impl FnOnce(&NamedType<Str, OriginalNode>) -> TSType,
) -> (TSType, bool) {
    match ty {
        Type::Named(name) => (mapper(name), true),
        Type::List(inner) => (TSType::Array(Box::new(map_to_tstype(inner, mapper))), true),
        Type::NonNull(inner) => {
            let (inner_ty, _) = map_to_tstype_impl(inner, mapper);
            (inner_ty, false)
        }
    }
}
