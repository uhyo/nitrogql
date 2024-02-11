use graphql_type_system::{NamedType, Text, Type};
use nitrogql_config_file::TypeTarget;

use crate::ts_types::{ts_types_util::ts_union, ObjectField, TSType};

use super::{SelectionTree, SelectionTreeField};

#[derive(Debug, Copy, Clone)]
pub struct GenerateSelectionTreeTypeContext<'a> {
    pub schema_root_namespace: &'a str,
}

/// Generate a TypeScript representation of the selection tree.
pub fn generate_selection_tree_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    tree: &SelectionTree<S>,
) -> TSType {
    generate_selection_tree_type_impl(context, tree, false)
}

fn generate_selection_tree_type_impl<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    tree: &SelectionTree<S>,
    is_non_null: bool,
) -> TSType {
    match tree {
        SelectionTree::NonNull(inner) => generate_selection_tree_type_impl(context, inner, true),
        SelectionTree::List(inner) => {
            let list_type = TSType::Array(Box::new(generate_selection_tree_type_impl(
                context, inner, false,
            )));
            if is_non_null {
                list_type
            } else {
                ts_union(vec![list_type, TSType::Null])
            }
        }
        SelectionTree::Object(branches) => {
            let branches_type = ts_union(branches.iter().map(|branch| {
                let selection_set_utility = TSType::NamespaceMember(
                    context.schema_root_namespace.into(),
                    "__SelectionSet".into(),
                );
                let schema_type = TSType::NamespaceMember3(
                    context.schema_root_namespace.into(),
                    TypeTarget::OperationOutput.to_string(),
                    branch.type_name.clone(),
                );

                let unaliased_object = TSType::Object(
                    branch
                        .unaliased_fields
                        .iter()
                        .map(|field| field_to_type(context, field))
                        .collect(),
                );
                let aliased_object = TSType::Object(
                    branch
                        .aliased_fields
                        .iter()
                        .map(|field| field_to_type(context, field))
                        .collect(),
                );
                TSType::TypeFunc(
                    Box::new(selection_set_utility),
                    vec![schema_type, unaliased_object, aliased_object],
                )
            }));
            if is_non_null {
                branches_type
            } else {
                ts_union(vec![branches_type, TSType::Null])
            }
        }
    }
}

fn field_to_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    field: &SelectionTreeField<S>,
) -> ObjectField {
    match field {
        SelectionTreeField::Empty(empty) => ObjectField {
            key: empty.name.to_string().into(),
            r#type: TSType::Never,
            description: None,
            optional: true,
            readonly: false,
        },
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
                readonly: false,
            }
        }
        SelectionTreeField::Object(object) => ObjectField {
            key: object.name.clone().to_string().into(),
            r#type: generate_selection_tree_type_impl(context, &object.selection, false),
            description: None,
            optional: false,
            readonly: false,
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
