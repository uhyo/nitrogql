use graphql_type_system::{NamedType, Schema, Text, Type};
use nitrogql_ast::base::Pos;
use nitrogql_config_file::TypeTarget;

use crate::{
    jsdoc::description_with_deprecation,
    ts_types::{ObjectField, TSType, ts_types_util::ts_union},
};

use super::{
    super::fragment_imports::{FragmentImport, UTILITIES_NAMESPACE, imported_fragment_tree},
    SelectionTree, SelectionTreeBranch, SelectionTreeField, SelectionTreeLeaf,
};

pub struct GenerateSelectionTreeTypeContext<'a, S> {
    /// Schema, whose descriptions of fields are copied to the printed types.
    pub schema: &'a Schema<S, Pos>,
    pub schema_root_namespace: &'a str,
    pub fragment_imports: &'a [FragmentImport],
}

/// Generate a TypeScript representation of the selection tree.
///
/// Trees that refer to imported fragments are printed unrendered,
/// and the fragments are merged and the result is rendered at the type level.
pub fn generate_selection_tree_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext<S>,
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

/// Generate the unrendered type of given branches, which is a record that maps names of object
/// types to `Branch` types (a union of them, if branches of a type differ in variable values).
/// Other modules merge this type into their selections.
pub fn generate_unrendered_branches_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext<S>,
    branches: &[SelectionTreeBranch<S>],
) -> TSType {
    let mut type_names: Vec<&str> = vec![];
    for branch in branches {
        if !type_names.contains(&branch.type_name.as_str()) {
            type_names.push(&branch.type_name);
        }
    }
    TSType::Object(
        type_names
            .into_iter()
            .map(|type_name| ObjectField {
                key: type_name.into(),
                r#type: ts_union(
                    branches
                        .iter()
                        .filter(|branch| branch.type_name == type_name)
                        .map(|branch| generate_unrendered_branch_type(context, branch)),
                ),
                description: None,
                optional: false,
                readonly: false,
            })
            .collect(),
    )
}

fn generate_unrendered_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext<S>,
    tree: &SelectionTree<S>,
) -> TSType {
    wrap_nullability(tree, false, &|branches| {
        generate_unrendered_branches_type(context, branches)
    })
}

fn generate_unrendered_branch_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext<S>,
    branch: &SelectionTreeBranch<S>,
) -> TSType {
    // Fields are classified by kind, so that merging does not inspect them.
    // Aliased fields are typed the same as unaliased ones, except when omitted.
    let fields = |is_kind: fn(&SelectionTreeField<S>) -> bool| {
        fields_to_object(
            context,
            branch,
            branch_fields(branch).filter(|(field, _)| is_kind(field)),
            |tree| generate_unrendered_type(context, tree),
        )
    };
    let names = |fields: &mut dyn Iterator<Item = &SelectionTreeField<S>>| {
        ts_union(fields.map(|field| TSType::StringLiteral(field.name().to_string())))
    };
    let leaves = fields(|field| matches!(field, SelectionTreeField::Leaf(_)));
    let objects = fields(|field| matches!(field, SelectionTreeField::Object(_)));
    let omitted = names(
        &mut branch_fields(branch)
            .map(|(field, _)| field)
            .filter(|field| matches!(field, SelectionTreeField::Empty(_))),
    );
    let aliased = names(&mut branch.aliased_fields.iter());

    // Branches of this object type in the spread fragments.
    let type_name = TSType::StringLiteral(branch.type_name.clone());
    let fragments = combine_fragment_trees(
        branch
            .fragment_refs
            .iter()
            .map(|fragment_ref| {
                let tree = TSType::Raw(imported_fragment_tree(
                    context.fragment_imports,
                    &fragment_ref.name.to_string(),
                ));
                let branches =
                    TSType::TypeFunc(Box::new(utility("At")), vec![tree, type_name.clone()]);
                if fragment_ref.skipped {
                    TSType::TypeFunc(Box::new(utility("SkipFragment")), vec![branches])
                } else {
                    branches
                }
            })
            .collect(),
    );
    // Fragments are merged into branches that have only leaves with simpler utilities,
    // as parts that the branch does not have need not be merged.
    let only_leaves = matches!(&objects, TSType::Object(fields) if fields.is_empty())
        && branch.variables.is_empty();
    let is_empty = only_leaves
        && matches!(&leaves, TSType::Object(fields) if fields.is_empty())
        && omitted.is_never();
    match fragments {
        Some(fragments) if is_empty => {
            TSType::TypeFunc(Box::new(utility("Spread")), vec![fragments])
        }
        Some(fragments) if only_leaves => TSType::TypeFunc(
            Box::new(utility("MergeLeaves")),
            vec![fragments, leaves, omitted, aliased],
        ),
        fragments => {
            let mut branch_args = vec![leaves, objects, omitted, aliased];
            // Values of variables, so that merging skips branches computed with other values.
            if !branch.variables.is_empty() {
                branch_args.push(ts_union(
                    branch
                        .variables
                        .iter()
                        .map(|(name, value)| TSType::StringLiteral(format!("{name}={value}"))),
                ));
            }
            let branch_type = TSType::TypeFunc(Box::new(utility("Branch")), branch_args);
            match fragments {
                Some(fragments) => TSType::TypeFunc(
                    Box::new(utility("MergeFragment")),
                    vec![branch_type, fragments],
                ),
                None => branch_type,
            }
        }
    }
}

/// Combines branches of fragments into one.
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
    context: &GenerateSelectionTreeTypeContext<S>,
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

/// Object type of given branch.
/// Fields are typed by the printer instead of being picked from the schema type, which
/// costs type checking in proportion to the number of fields of the schema type.
fn generate_selection_set_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext<S>,
    branch: &SelectionTreeBranch<S>,
) -> TSType {
    fields_to_object(context, branch, branch_fields(branch), |tree| {
        generate_selection_tree_type_impl(context, tree)
    })
}

/// Fields of given branch, with whether each is aliased.
fn branch_fields<S>(
    branch: &SelectionTreeBranch<S>,
) -> impl Iterator<Item = (&SelectionTreeField<S>, bool)> {
    let unaliased = branch.unaliased_fields.iter().map(|field| (field, false));
    let aliased = branch.aliased_fields.iter().map(|field| (field, true));
    unaliased.chain(aliased)
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

/// Object type of given fields selected in given branch, with `object` giving types of object
/// fields. Unaliased fields have the descriptions of the fields in the schema.
fn fields_to_object<'a, 'src, S: Text<'src> + 'a>(
    context: &GenerateSelectionTreeTypeContext<S>,
    branch: &SelectionTreeBranch<S>,
    fields: impl Iterator<Item = (&'a SelectionTreeField<S>, bool)>,
    object: impl Fn(&SelectionTree<S>) -> TSType,
) -> TSType {
    let schema_fields = context
        .schema
        .get_type(&branch.type_name)
        .and_then(|ty| ty.as_object())
        .map_or(&[][..], |ty| &ty.fields);
    TSType::Object(
        fields
            .map(|(field, aliased)| {
                let (r#type, optional) = match field {
                    // Omitted fields are typed as `undefined` (`never` with
                    // `exactOptionalPropertyTypes`), or as optional `never` if aliased.
                    SelectionTreeField::Empty(_) if aliased => (TSType::Never, true),
                    SelectionTreeField::Empty(_) => (
                        TSType::NamespaceMember(
                            context.schema_root_namespace.into(),
                            "__Omitted".into(),
                        ),
                        false,
                    ),
                    SelectionTreeField::Leaf(leaf) => {
                        (leaf_type(context, leaf, &branch.type_name), false)
                    }
                    SelectionTreeField::Object(field) => (object(&field.selection), false),
                };
                let schema_field = schema_fields
                    .iter()
                    .find(|schema_field| !aliased && *schema_field.name == *field.name());
                ObjectField {
                    key: field.name().to_string().into(),
                    r#type,
                    description: schema_field.and_then(|schema_field| {
                        description_with_deprecation(
                            schema_field
                                .description
                                .as_ref()
                                .map(|d| d.inner_ref().borrow()),
                            schema_field.deprecation.as_ref(),
                        )
                    }),
                    optional,
                    readonly: false,
                }
            })
            .collect(),
    )
}

/// Type of given GraphQL type in the schema's `__OperationOutput` namespace.
fn operation_output_type<S>(
    context: &GenerateSelectionTreeTypeContext<S>,
    type_name: String,
) -> TSType {
    TSType::NamespaceMember3(
        context.schema_root_namespace.into(),
        TypeTarget::OperationOutput.to_string(),
        type_name,
    )
}

fn leaf_type<'src, S: Text<'src>>(
    context: &GenerateSelectionTreeTypeContext<S>,
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
