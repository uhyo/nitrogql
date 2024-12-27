use std::{collections::HashMap, convert::identity, iter::once};

use crate::{
    ts_types::{ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type, ObjectField},
    utils::interface_implementers,
};
use graphql_type_system::{NamedType, Node, ObjectDefinition, Schema, Text, Type, TypeDefinition};
use itertools::{Either, Itertools};
use nitrogql_ast::{
    base::Pos,
    directive::Directive,
    operation::FragmentDefinition,
    selection_set::{Selection, SelectionSet},
    value::Value,
    variable::VariablesDefinition,
};
use nitrogql_config_file::TypeTarget;
use nitrogql_semantics::direct_fields_of_output_type;

use super::{
    super::ts_types::{ts_types_util::ts_intersection, TSType},
    branching::BranchingCondition,
    deep_merge::deep_merge_selection_tree,
    selection_set_visitor::visit_fields_in_selection_set,
    selection_tree::{
        SelectionTree, SelectionTreeBranch, SelectionTreeEmptyLeaf, SelectionTreeField,
        SelectionTreeLeaf, SelectionTreeObject,
    },
    visitor::OperationTypePrinterOptions,
};

pub struct QueryTypePrinterContext<'a, 'src, S: Text<'src>> {
    pub options: &'a OperationTypePrinterOptions,
    pub schema: &'a Schema<S, Pos>,
    pub fragment_definitions: &'a HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

pub fn get_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    parent_type: &Type<S, Pos>,
) -> SelectionTree<S> {
    type_to_selection_tree(parent_type, |parent_type| {
        let branches = generate_branching_conditions(context, selection_set, parent_type);
        branches
            .into_iter()
            .map(|branch| get_object_type_for_selection_set(context, selection_set, &branch))
            .collect()
    })
}

fn type_to_selection_tree<S, F: FnOnce(&NamedType<S, Pos>) -> Vec<SelectionTreeBranch<S>>>(
    ty: &Type<S, Pos>,
    mapper: F,
) -> SelectionTree<S> {
    match ty {
        Type::Named(ref name) => SelectionTree::Object(mapper(name)),
        Type::List(ref inner) => {
            SelectionTree::List(Box::new(type_to_selection_tree(inner, mapper)))
        }
        Type::NonNull(ref inner) => {
            SelectionTree::NonNull(Box::new(type_to_selection_tree(inner, mapper)))
        }
    }
}

/// Generates a set of branching conditions for a given selection set.
fn generate_branching_conditions<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'src>,
    parent_type: &'a NamedType<S, Pos>,
) -> Vec<BranchingCondition<'a, S>> {
    let parent_type_def = context
        .schema
        .get_type(parent_type)
        .expect("Type system error");
    let parent_objects = match **parent_type_def {
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            panic!("Type system error")
        }
        TypeDefinition::Object(ref obj_def) => {
            vec![obj_def]
        }
        TypeDefinition::Interface(ref interface_def) => {
            let object_defs = interface_implementers(context.schema, &interface_def.name);
            object_defs.collect()
        }
        TypeDefinition::Union(ref union_def) => {
            let object_defs = union_def.possible_types.iter().map(|member| {
                context
                    .schema
                    .get_type(member)
                    .and_then(|def| def.as_object())
                    .expect("Type system error")
            });
            object_defs.collect()
        }
    };
    // multi_cartesian_product cannot handle the case where there are no variables.
    // See: https://github.com/rust-itertools/itertools/issues/337
    let boolean_variables = get_boolean_variables(context, selection_set);
    let boolean_variables = if boolean_variables.is_empty() {
        Either::Left(once(vec![]))
    } else {
        Either::Right(
            boolean_variables
                .into_iter()
                .unique()
                .map(|v| vec![(v, false), (v, true)])
                .multi_cartesian_product(),
        )
    };
    parent_objects
        .into_iter()
        .cartesian_product(boolean_variables)
        .map(|(obj, vars)| BranchingCondition {
            parent_obj: obj,
            boolean_variables: vars,
        })
        .collect()
}

/// Get boolean variables involved in a selection set.
fn get_boolean_variables<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
) -> Vec<&'src str> {
    let mut variables = Vec::new();
    visit_fields_in_selection_set(context, selection_set, |selection| {
        let directives = selection.directives();
        for directive in directives {
            if directive.name.name == "skip" || directive.name.name == "include" {
                let variable = directive
                    .arguments
                    .iter()
                    .flatten()
                    .find_map(|(arg, value)| {
                        if arg.name != "if" {
                            return None;
                        }
                        if let Value::Variable(ref v) = value {
                            Some(v.name)
                        } else {
                            None
                        }
                    });
                if let Some(variable) = variable {
                    variables.push(variable);
                }
            }
        }
    });
    variables
}

fn get_object_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    branch: &BranchingCondition<S>,
) -> SelectionTreeBranch<S> {
    let (unaliased, aliased): (Vec<_>, Vec<_>) =
        get_fields_for_selection_set(context, selection_set, branch)
            .into_iter()
            .partition_map(identity);
    let unaliased = deep_merge_selection_tree(unaliased);
    let aliased = deep_merge_selection_tree(aliased);
    SelectionTreeBranch {
        type_name: branch.parent_obj.name.to_string(),
        unaliased_fields: unaliased,
        aliased_fields: aliased,
    }
}

/// Returns an iterator of object fields.
/// Left is for non-aliased fields.
/// Right is for aliased fields.
fn get_fields_for_selection_set<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'src>,
    branch: &'a BranchingCondition<'a, S>,
) -> Vec<Either<SelectionTreeField<S>, SelectionTreeField<S>>> {
    let parent_type_def = context
        .schema
        .get_type(&branch.parent_obj.name)
        .expect("Type system error");

    let parent_fields = direct_fields_of_output_type(parent_type_def).expect("Type system error");

    let types_for_simple_fields =
        selection_set
            .selections
            .iter()
            .filter_map(move |sel| match sel {
                Selection::Field(ref field) => {
                    let field_name = field.name.name;
                    let selection_key =
                        field.alias.map(|name| name.name).unwrap_or(field.name.name);
                    let field_type = if check_skip_directive(branch, &field.directives) {
                        SelectionTreeField::Empty(SelectionTreeEmptyLeaf {
                            name: selection_key.into(),
                        })
                    } else if field_name == "__typename" {
                        // Special handling of reflection
                        SelectionTreeField::Leaf(SelectionTreeLeaf {
                            name: selection_key.into(),
                            r#type: Type::Named(NamedType::from(Node::from(
                                S::from("String"),
                                Pos::builtin(),
                            ))),
                        })
                    } else {
                        let field_def = parent_fields
                            .iter()
                            .find(|parent_field| {
                                parent_field.name.inner_ref().borrow() == field_name
                            })
                            .expect("Type system error");

                        match field.selection_set {
                            None => SelectionTreeField::Leaf(SelectionTreeLeaf {
                                name: selection_key.into(),
                                r#type: field_def.r#type.clone(),
                            }),
                            Some(ref selection_set) => {
                                let object_type = get_type_for_selection_set(
                                    context,
                                    selection_set,
                                    &field_def.r#type,
                                );
                                SelectionTreeField::Object(SelectionTreeObject {
                                    name: selection_key.into(),
                                    selection: object_type,
                                })
                            }
                        }
                    };

                    match field.alias {
                        None => Some(Either::Left(field_type)),
                        Some(_) => Some(Either::Right(field_type)),
                    }
                }
                _ => None,
            });

    let types_for_fragments = selection_set
        .selections
        .iter()
        .flat_map(move |sel| match sel {
            Selection::Field(_) => vec![],
            Selection::FragmentSpread(ref fragment) => {
                let fragment_def = context
                    .fragment_definitions
                    .get(fragment.fragment_name.name)
                    .expect("Type system error");
                if check_fragment_condition(
                    context,
                    branch.parent_obj,
                    fragment_def.type_condition.name,
                ) {
                    let fields =
                        get_fields_for_selection_set(context, &fragment_def.selection_set, branch);
                    if check_skip_directive(branch, &fragment.directives) {
                        fields
                            .into_iter()
                            .map(|field| {
                                field.map(|field| {
                                    SelectionTreeField::Empty(SelectionTreeEmptyLeaf {
                                        name: field.name().clone(),
                                    })
                                })
                            })
                            .collect()
                    } else {
                        fields
                    }
                } else {
                    vec![]
                }
            }
            Selection::InlineFragment(ref fragment) => match fragment.type_condition {
                None => {
                    let fields =
                        get_fields_for_selection_set(context, &fragment.selection_set, branch);
                    if check_skip_directive(branch, &fragment.directives) {
                        fields
                            .into_iter()
                            .map(|field| {
                                field.map(|field| {
                                    SelectionTreeField::Empty(SelectionTreeEmptyLeaf {
                                        name: field.name().clone(),
                                    })
                                })
                            })
                            .collect()
                    } else {
                        fields
                    }
                }
                Some(ref cond) => {
                    if check_fragment_condition(context, branch.parent_obj, cond.name) {
                        let fields =
                            get_fields_for_selection_set(context, &fragment.selection_set, branch);
                        if check_skip_directive(branch, &fragment.directives) {
                            fields
                                .into_iter()
                                .map(|field| {
                                    field.map(|field| {
                                        SelectionTreeField::Empty(SelectionTreeEmptyLeaf {
                                            name: field.name().clone(),
                                        })
                                    })
                                })
                                .collect()
                        } else {
                            fields
                        }
                    } else {
                        vec![]
                    }
                }
            },
        });
    types_for_simple_fields
        .chain(types_for_fragments)
        .collect::<Vec<_>>()
}

/// Examine directives and returns whether field should be skipped.
fn check_skip_directive<'src, S: Text<'src>>(
    branch: &BranchingCondition<S>,
    directives: &[Directive<'src>],
) -> bool {
    for directive in directives {
        match directive.name.name {
            "skip" => {
                let (_, skip) = directive
                    .arguments
                    .iter()
                    .flatten()
                    .find(|(arg, _)| arg.name == "if")
                    .expect("Type system error");
                match skip {
                    Value::Variable(var) => {
                        let (_, var_value) = branch
                            .boolean_variables
                            .iter()
                            .find(|(name, _)| *name == var.name)
                            .expect("Type system error");
                        if *var_value {
                            return true;
                        }
                    }
                    Value::BooleanValue(b) => {
                        if b.value {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            "include" => {
                let (_, include) = directive
                    .arguments
                    .iter()
                    .flatten()
                    .find(|(arg, _)| arg.name == "if")
                    .expect("Type system error");
                match include {
                    Value::Variable(var) => {
                        let (_, var_value) = branch
                            .boolean_variables
                            .iter()
                            .find(|(name, _)| *name == var.name)
                            .expect("Type system error");
                        if !var_value {
                            return true;
                        }
                    }
                    Value::BooleanValue(b) => {
                        if !b.value {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    false
}

/// Returns whether given object type implements given condition.
fn check_fragment_condition<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    object_def: &ObjectDefinition<S, Pos>,
    cond: &str,
) -> bool {
    let cond_type = context.schema.get_type(cond).expect("Type system error");
    match **cond_type {
        TypeDefinition::Object(ref obj) => object_def.name.inner_ref() == &*obj.name,
        TypeDefinition::Interface(ref interface) => object_def
            .interfaces
            .iter()
            .any(|imp| imp.inner_ref() == &*interface.name),
        TypeDefinition::Union(ref union) => union
            .possible_types
            .iter()
            .any(|mem| mem.inner_ref() == &*object_def.name),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            false
        }
    }
}

pub fn get_type_for_variable_definitions<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    definitions: &VariablesDefinition,
) -> TSType {
    let types_for_each_field: Vec<_> = definitions
        .definitions
        .iter()
        .map(|def| {
            let property_name = def.name.name;
            let field_type = get_ts_type_of_type(&def.r#type, |name| {
                TSType::NamespaceMember3(
                    context.options.schema_root_namespace.as_str().into(),
                    TypeTarget::OperationInput.to_string(),
                    name.name.to_string(),
                )
            });
            let is_optional =
                !def.r#type.is_nonnull() && context.options.allow_undefined_as_optional_input;
            let field_type = if is_optional {
                ts_union(vec![field_type, TSType::Undefined])
            } else {
                field_type
            };
            TSType::Object(vec![ObjectField {
                key: property_name.into(),
                optional: is_optional,
                r#type: field_type,
                readonly: true,
                description: None,
            }])
        })
        .collect();

    if types_for_each_field.is_empty() {
        TSType::empty_object()
    } else {
        ts_intersection(types_for_each_field)
    }
}
