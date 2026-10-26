use std::{collections::HashMap, convert::identity, iter::once};

use crate::{
    ts_types::{ObjectField, ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type},
    utils::interface_implementers,
};
use graphql_type_system::{NamedType, Node, ObjectDefinition, Schema, Text, Type, TypeDefinition};
use itertools::{Either, Itertools};
use nitrogql_ast::{
    base::{Ident, Pos},
    directive::Directive,
    operation::FragmentDefinition,
    selection_set::{Selection, SelectionSet},
    value::Value,
    variable::VariablesDefinition,
};
use nitrogql_config_file::TypeTarget;
use nitrogql_semantics::direct_fields_of_output_type;

use super::{
    super::ts_types::{TSType, ts_types_util::ts_intersection},
    branching::BranchingCondition,
    deep_merge::{FieldShapeMismatch, deep_merge_selection_tree},
    error::{OperationTypePrinterErrorMessage, OperationTypePrinterResult},
    selection_set_visitor::visit_fields_in_selection_set,
    selection_tree::{
        FragmentRef, SelectionTree, SelectionTreeBranch, SelectionTreeEmptyLeaf,
        SelectionTreeField, SelectionTreeLeaf, SelectionTreeObject,
    },
    visitor::OperationTypePrinterOptions,
};

pub struct QueryTypePrinterContext<'a, 'src, S: Text<'src>> {
    pub options: &'a OperationTypePrinterOptions,
    pub schema: &'a Schema<S, Pos>,
    pub fragment_definitions: &'a HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

impl<'src, S: Text<'src>> QueryTypePrinterContext<'_, 'src, S> {
    /// Returns whether given fragment is defined in another module.
    pub fn is_external_fragment(&self, name: &str) -> bool {
        !self.fragment_definitions.contains_key(name)
            && self.options.external_fragments.contains_key(name)
    }
}

/// Fields selected by a selection set under a branching condition.
struct SelectedFields<S> {
    /// Left is for non-aliased fields. Right is for aliased fields.
    fields: Vec<Either<SelectionTreeField<S>, SelectionTreeField<S>>>,
    /// Spreads of fragments defined in other modules.
    fragment_refs: Vec<FragmentRef<S>>,
}

pub fn get_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    parent_type: &Type<S, Pos>,
) -> OperationTypePrinterResult<SelectionTree<S>> {
    type_to_selection_tree(parent_type, |parent_type| {
        let branches = generate_branching_conditions(context, selection_set, parent_type)?;
        branches
            .into_iter()
            .map(|branch| get_object_type_for_selection_set(context, selection_set, &branch))
            .collect()
    })
}

fn type_to_selection_tree<
    S,
    F: FnOnce(&NamedType<S, Pos>) -> OperationTypePrinterResult<Vec<SelectionTreeBranch<S>>>,
>(
    ty: &Type<S, Pos>,
    mapper: F,
) -> OperationTypePrinterResult<SelectionTree<S>> {
    Ok(match ty {
        Type::Named(name) => SelectionTree::Object(mapper(name)?),
        Type::List(inner) => SelectionTree::List(Box::new(type_to_selection_tree(inner, mapper)?)),
        Type::NonNull(inner) => {
            SelectionTree::NonNull(Box::new(type_to_selection_tree(inner, mapper)?))
        }
    })
}

/// Generates a set of branching conditions for a given selection set.
fn generate_branching_conditions<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'src>,
    parent_type: &'a NamedType<S, Pos>,
) -> OperationTypePrinterResult<Vec<BranchingCondition<'a, S>>> {
    let parent_type_def = context.schema.get_type(parent_type).ok_or_else(|| {
        OperationTypePrinterErrorMessage::UnknownType {
            name: parent_type.to_string(),
        }
        .with_pos(selection_set.position)
    })?;
    let parent_objects = match **parent_type_def {
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            return Err(OperationTypePrinterErrorMessage::NotSelectable {
                name: parent_type.to_string(),
            }
            .with_pos(selection_set.position));
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
                    .ok_or_else(|| {
                        OperationTypePrinterErrorMessage::UnionMemberNotObject {
                            name: member.to_string(),
                        }
                        .with_pos(selection_set.position)
                    })
            });
            object_defs.collect::<Result<_, _>>()?
        }
    };
    // multi_cartesian_product cannot handle the case where there are no variables.
    // See: https://github.com/rust-itertools/itertools/issues/337
    let boolean_variables = get_boolean_variables(context, selection_set)?;
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
    Ok(parent_objects
        .into_iter()
        .cartesian_product(boolean_variables)
        .map(|(obj, vars)| BranchingCondition {
            parent_obj: obj,
            boolean_variables: vars,
        })
        .collect())
}

/// Get boolean variables involved in a selection set.
fn get_boolean_variables<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
) -> OperationTypePrinterResult<Vec<&'src str>> {
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
                        if let Value::Variable(v) = value {
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
    })?;
    Ok(variables)
}

fn get_object_type_for_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    branch: &BranchingCondition<S>,
) -> OperationTypePrinterResult<SelectionTreeBranch<S>> {
    let selected = get_fields_for_selection_set(context, selection_set, branch)?;
    let (unaliased, aliased): (Vec<_>, Vec<_>) =
        selected.fields.into_iter().partition_map(identity);
    let shape_mismatch = |err: FieldShapeMismatch<S>| {
        // Point at the first field of the conflicting key if it is directly in this selection set.
        let position = selection_set
            .selections
            .iter()
            .find_map(|sel| match sel {
                Selection::Field(field)
                    if field.alias.unwrap_or(field.name).name == err.key.borrow() =>
                {
                    Some(field.alias.unwrap_or(field.name).position)
                }
                _ => None,
            })
            .unwrap_or(selection_set.position);
        OperationTypePrinterErrorMessage::FieldShapeMismatch {
            name: err.name.to_string(),
        }
        .with_pos(position)
    };
    let unaliased = deep_merge_selection_tree(unaliased).map_err(shape_mismatch)?;
    let aliased = deep_merge_selection_tree(aliased).map_err(shape_mismatch)?;
    Ok(SelectionTreeBranch {
        type_name: branch.parent_obj.name.to_string(),
        unaliased_fields: unaliased,
        aliased_fields: aliased,
        fragment_refs: selected.fragment_refs,
    })
}

/// Returns fields selected by given selection set.
fn get_fields_for_selection_set<'a, 'src, S: Text<'src>>(
    context: &'a QueryTypePrinterContext<'a, 'src, S>,
    selection_set: &'a SelectionSet<'src>,
    branch: &'a BranchingCondition<'a, S>,
) -> OperationTypePrinterResult<SelectedFields<S>> {
    let parent_type_name = &branch.parent_obj.name;
    let parent_fields = context
        .schema
        .get_type(parent_type_name)
        .and_then(|def| direct_fields_of_output_type(def))
        .ok_or_else(|| {
            OperationTypePrinterErrorMessage::UnknownType {
                name: parent_type_name.to_string(),
            }
            .with_pos(selection_set.position)
        })?;

    let mut types_for_simple_fields = vec![];
    for sel in &selection_set.selections {
        let Selection::Field(field) = sel else {
            continue;
        };
        let field_name = field.name.name;
        let selection_key = field.alias.map(|name| name.name).unwrap_or(field.name.name);
        let field_type = if check_skip_directive(branch, &field.directives)? {
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
                .find(|parent_field| parent_field.name.inner_ref().borrow() == field_name)
                .ok_or_else(|| {
                    OperationTypePrinterErrorMessage::UnknownField {
                        field: field_name.to_owned(),
                        type_name: parent_type_name.to_string(),
                    }
                    .with_pos(field.name.position)
                })?;

            match field.selection_set {
                None => SelectionTreeField::Leaf(SelectionTreeLeaf {
                    name: selection_key.into(),
                    r#type: field_def.r#type.clone(),
                }),
                Some(ref selection_set) => {
                    let object_type =
                        get_type_for_selection_set(context, selection_set, &field_def.r#type)?;
                    SelectionTreeField::Object(SelectionTreeObject {
                        name: selection_key.into(),
                        selection: object_type,
                    })
                }
            }
        };

        types_for_simple_fields.push(match field.alias {
            None => Either::Left(field_type),
            Some(_) => Either::Right(field_type),
        });
    }

    let mut types_for_fragments = vec![];
    let mut fragment_refs = vec![];
    for sel in &selection_set.selections {
        let (type_condition, fragment_selection_set, directives) = match sel {
            Selection::Field(_) => continue,
            Selection::FragmentSpread(fragment)
                if context.is_external_fragment(fragment.fragment_name.name) =>
            {
                // Type condition is checked at the type level,
                // as the fragment's selection has no branch for other types.
                fragment_refs.push(FragmentRef {
                    name: fragment.fragment_name.name.into(),
                    skipped: check_skip_directive(branch, &fragment.directives)?,
                });
                continue;
            }
            Selection::FragmentSpread(fragment) => {
                let fragment_def = context
                    .fragment_definitions
                    .get(fragment.fragment_name.name)
                    .ok_or_else(|| {
                        OperationTypePrinterErrorMessage::UnknownFragment {
                            name: fragment.fragment_name.name.to_owned(),
                        }
                        .with_pos(fragment.fragment_name.position)
                    })?;
                (
                    Some(&fragment_def.type_condition),
                    &fragment_def.selection_set,
                    &fragment.directives,
                )
            }
            Selection::InlineFragment(fragment) => (
                fragment.type_condition.as_ref(),
                &fragment.selection_set,
                &fragment.directives,
            ),
        };
        if let Some(cond) = type_condition
            && !check_fragment_condition(context, branch.parent_obj, cond)?
        {
            continue;
        }
        let SelectedFields {
            fields,
            fragment_refs: nested_refs,
        } = get_fields_for_selection_set(context, fragment_selection_set, branch)?;
        if check_skip_directive(branch, directives)? {
            fragment_refs.extend(nested_refs.into_iter().map(|fragment_ref| FragmentRef {
                skipped: true,
                ..fragment_ref
            }));
            types_for_fragments.extend(fields.into_iter().map(|field| {
                field.map(|field| {
                    SelectionTreeField::Empty(SelectionTreeEmptyLeaf {
                        name: field.name().clone(),
                    })
                })
            }));
        } else {
            fragment_refs.extend(nested_refs);
            types_for_fragments.extend(fields);
        }
    }
    types_for_simple_fields.extend(types_for_fragments);
    Ok(SelectedFields {
        fields: types_for_simple_fields,
        fragment_refs,
    })
}

/// Examine directives and returns whether field should be skipped.
fn check_skip_directive<'src, S: Text<'src>>(
    branch: &BranchingCondition<S>,
    directives: &[Directive<'src>],
) -> OperationTypePrinterResult<bool> {
    for directive in directives {
        // `@skip` skips when the condition is true, `@include` skips when it is false.
        let skip_when = match directive.name.name {
            "skip" => true,
            "include" => false,
            _ => continue,
        };
        let (_, condition) = directive
            .arguments
            .iter()
            .flatten()
            .find(|(arg, _)| arg.name == "if")
            .ok_or_else(|| {
                OperationTypePrinterErrorMessage::MissingIfArgument {
                    name: directive.name.name.to_owned(),
                }
                .with_pos(directive.position)
            })?;
        let condition = match condition {
            Value::Variable(var) => {
                let (_, var_value) = branch
                    .boolean_variables
                    .iter()
                    .find(|(name, _)| *name == var.name)
                    .ok_or_else(|| {
                        OperationTypePrinterErrorMessage::UnknownVariable {
                            name: var.name.to_owned(),
                        }
                        .with_pos(directive.position)
                    })?;
                *var_value
            }
            Value::BooleanValue(b) => b.value,
            _ => continue,
        };
        if condition == skip_when {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Returns whether given object type implements given condition.
fn check_fragment_condition<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    object_def: &ObjectDefinition<S, Pos>,
    cond: &Ident<'src>,
) -> OperationTypePrinterResult<bool> {
    let cond_type = context.schema.get_type(cond.name).ok_or_else(|| {
        OperationTypePrinterErrorMessage::UnknownType {
            name: cond.name.to_owned(),
        }
        .with_pos(cond.position)
    })?;
    Ok(match **cond_type {
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
    })
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
                description: def.description.as_ref().map(|d| d.value.clone()),
            }])
        })
        .collect();

    if types_for_each_field.is_empty() {
        TSType::empty_object()
    } else {
        ts_intersection(types_for_each_field)
    }
}
