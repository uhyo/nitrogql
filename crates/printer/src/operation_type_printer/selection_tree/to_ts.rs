use graphql_type_system::{NamedType, Text, Type};
use nitrogql_config_file::TypeTarget;

use crate::ts_types::{ObjectField, TSType, ts_types_util::ts_union};

use super::{
    super::fragment_imports::{FragmentImport, UTILITIES_NAMESPACE, imported_fragment_tree},
    SelectionTree, SelectionTreeBranch, SelectionTreeField, SelectionTreeLeaf,
};

#[derive(Debug, Copy, Clone)]
pub struct GenerateSelectionTreeTypeContext<'a> {
    pub schema_root_namespace: &'a str,
    pub fragment_imports: &'a [FragmentImport],
}

/// Generate a TypeScript representation of the selection tree.
///
/// Trees that refer to imported fragments are printed unrendered,
/// and the fragments are merged and the result is rendered at the type level.
pub fn generate_selection_tree_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    tree: &SelectionTree<S>,
) -> TSType {
    if tree.has_fragment_refs() {
        return render_type(generate_unrendered_type(context, tree));
    }
    generate_selection_tree_type_impl(context, tree)
}

/// Renders given unrendered type at the type level.
pub fn render_type(unrendered: TSType) -> TSType {
    TSType::TypeFunc(Box::new(utility("Render")), vec![unrendered])
}

/// Generate the unrendered type of given branches, which is a union of `Branch` types.
/// Other modules merge this type into their selections.
pub fn generate_unrendered_branches_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    branches: &[SelectionTreeBranch<S>],
) -> TSType {
    ts_union(
        branches
            .iter()
            .map(|branch| generate_unrendered_branch_type(context, branch)),
    )
}

fn generate_unrendered_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    tree: &SelectionTree<S>,
) -> TSType {
    wrap_nullability(tree, false, &|branches| {
        generate_unrendered_branches_type(context, branches)
    })
}

fn generate_unrendered_branch_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    branch: &SelectionTreeBranch<S>,
) -> TSType {
    let fields = |fields: &[SelectionTreeField<S>]| {
        fields_to_object(context, fields, &branch.type_name, |tree| {
            generate_unrendered_type(context, tree)
        })
    };
    let mut branch_args = vec![
        TSType::StringLiteral(branch.type_name.clone()),
        operation_output_type(context, branch.type_name.clone()),
        fields(&branch.unaliased_fields),
        fields(&branch.aliased_fields),
    ];
    // Values of variables, so that merging skips branches computed with other values.
    if !branch.variables.is_empty() {
        branch_args.push(TSType::Object(
            branch
                .variables
                .iter()
                .map(|(name, value)| ObjectField {
                    key: name.clone().into(),
                    r#type: TSType::Raw(value.to_string()),
                    description: None,
                    optional: false,
                    readonly: false,
                })
                .collect(),
        ));
    }
    let branch_type = TSType::TypeFunc(Box::new(utility("Branch")), branch_args);
    let trees = branch
        .fragment_refs
        .iter()
        .map(|fragment_ref| {
            let tree = TSType::Raw(imported_fragment_tree(
                context.fragment_imports,
                &fragment_ref.name.to_string(),
            ));
            if fragment_ref.skipped {
                TSType::TypeFunc(Box::new(utility("SkipFragment")), vec![tree])
            } else {
                tree
            }
        })
        .collect::<Vec<_>>();
    match combine_fragment_trees(trees) {
        Some(fragments) => TSType::TypeFunc(
            Box::new(utility("MergeFragment")),
            vec![branch_type, fragments],
        ),
        None => branch_type,
    }
}

/// Combines selection trees of fragments into one.
/// They are combined as a balanced tree, as TypeScript limits the depth of nested instantiations.
fn combine_fragment_trees(mut trees: Vec<TSType>) -> Option<TSType> {
    while trees.len() > 1 {
        let mut pairs = trees.into_iter();
        trees = std::iter::from_fn(|| {
            let left = pairs.next()?;
            Some(match pairs.next() {
                Some(right) => {
                    TSType::TypeFunc(Box::new(utility("CombineFragments")), vec![left, right])
                }
                None => left,
            })
        })
        .collect();
    }
    trees.pop()
}

/// Utility type for merging imported fragments.
fn utility(name: &str) -> TSType {
    TSType::NamespaceMember(UTILITIES_NAMESPACE.into(), name.into())
}

fn generate_selection_tree_type_impl<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    tree: &SelectionTree<S>,
) -> TSType {
    wrap_nullability(tree, false, &|branches| {
        ts_union(
            branches
                .iter()
                .map(|branch| generate_selection_set_type(context, branch)),
        )
    })
}

fn generate_selection_set_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    branch: &SelectionTreeBranch<S>,
) -> TSType {
    let selection_set_utility = TSType::NamespaceMember(
        context.schema_root_namespace.into(),
        "__SelectionSet".into(),
    );
    let fields = |fields: &[SelectionTreeField<S>]| {
        fields_to_object(context, fields, &branch.type_name, |tree| {
            generate_selection_tree_type_impl(context, tree)
        })
    };
    TSType::TypeFunc(
        Box::new(selection_set_utility),
        vec![
            operation_output_type(context, branch.type_name.clone()),
            fields(&branch.unaliased_fields),
            fields(&branch.aliased_fields),
        ],
    )
}

/// Maps given tree to a type, with `object` giving types of objects.
/// Lists become arrays, and trees not wrapped in `NonNull` are nullable.
fn wrap_nullability<S>(
    tree: &SelectionTree<S>,
    is_non_null: bool,
    object: &impl Fn(&[SelectionTreeBranch<S>]) -> TSType,
) -> TSType {
    let result = match tree {
        SelectionTree::NonNull(inner) => return wrap_nullability(inner, true, object),
        SelectionTree::List(inner) => {
            TSType::Array(Box::new(wrap_nullability(inner, false, object)))
        }
        SelectionTree::Object(branches) => object(branches),
    };
    if is_non_null {
        result
    } else {
        ts_union(vec![result, TSType::Null])
    }
}

/// Object type of given selected fields, with `object` giving types of object fields.
fn fields_to_object<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    fields: &[SelectionTreeField<S>],
    parent_type_name: &str,
    object: impl Fn(&SelectionTree<S>) -> TSType,
) -> TSType {
    TSType::Object(
        fields
            .iter()
            .map(|field| {
                let (r#type, optional) = match field {
                    // Omitted fields are optional `never`.
                    SelectionTreeField::Empty(_) => (TSType::Never, true),
                    SelectionTreeField::Leaf(leaf) => {
                        (leaf_type(context, leaf, parent_type_name), false)
                    }
                    SelectionTreeField::Object(field) => (object(&field.selection), false),
                };
                ObjectField {
                    key: field.name().to_string().into(),
                    r#type,
                    description: None,
                    optional,
                    readonly: false,
                }
            })
            .collect(),
    )
}

/// Type of given GraphQL type in the schema's `__OperationOutput` namespace.
fn operation_output_type(context: &GenerateSelectionTreeTypeContext, type_name: String) -> TSType {
    TSType::NamespaceMember3(
        context.schema_root_namespace.into(),
        TypeTarget::OperationOutput.to_string(),
        type_name,
    )
}

fn leaf_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext,
    leaf: &SelectionTreeLeaf<S>,
    parent_type_name: &str,
) -> TSType {
    if leaf.name == "__typename" {
        // special case for __typename
        TSType::StringLiteral(parent_type_name.to_string())
    } else {
        map_to_tstype(&leaf.r#type, |ty| {
            operation_output_type(context, ty.to_string())
        })
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
