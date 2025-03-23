use std::borrow::Borrow;

use graphql_type_system::{Field, Node, OriginalNodeRef, RootTypes, Text, TypeDefinition};
use nitrogql_ast::{
    base::{HasPos, Pos},
    operation::{
        ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationDocument,
        OperationType,
    },
    selection_set::{
        Field as SelectionField, FragmentSpread, InlineFragment, Selection, SelectionSet,
    },
    variable::VariablesDefinition,
};

use self::{
    count_selection_set_fields::selection_set_has_more_than_one_fields,
    fragment_map::{generate_fragment_map, FragmentMap},
};

use super::{
    common::{check_arguments, check_directives},
    error::{CheckError, CheckErrorMessage, TypeKind},
    types::inout_kind_of_type,
};
use nitrogql_semantics::direct_fields_of_output_type;

mod context;
mod count_selection_set_fields;
mod fragment_map;
#[cfg(test)]
mod tests;

pub use context::OperationCheckContext;

pub fn check_operation_document<'src, S: Text<'src>>(
    document: &OperationDocument<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
) -> Vec<CheckError> {
    let mut result = vec![];

    let fragment_map = generate_fragment_map(document);

    let operation_num = document
        .definitions
        .iter()
        .filter(|def| matches!(def, ExecutableDefinition::OperationDefinition(_)))
        .count();

    for (idx, def) in document.definitions.iter().enumerate() {
        match def {
            ExecutableDefinition::OperationDefinition(op) => {
                match op.name {
                    None => {
                        // Unnamed operation must be the only operation in the document
                        if operation_num != 1 {
                            result.push(
                                CheckErrorMessage::UnNamedOperationMustBeSingle
                                    .with_pos(op.position),
                            );
                        }
                    }
                    Some(ref name) => {
                        // Find other one with same name
                        let dup = document
                            .definitions
                            .iter()
                            .take(idx)
                            .find(|other| match other {
                                ExecutableDefinition::OperationDefinition(def) => {
                                    def.name.is_some_and(|n| n.name == name.name)
                                }
                                ExecutableDefinition::FragmentDefinition(_) => false,
                            });
                        if let Some(other) = dup {
                            result.push(
                                CheckErrorMessage::DuplicateOperationName {
                                    operation_type: op.operation_type,
                                }
                                .with_pos(name.position)
                                .with_additional_info(vec![(
                                    *other.position(),
                                    CheckErrorMessage::AnotherDefinitionPos {
                                        name: name.to_string(),
                                    },
                                )]),
                            );
                        }
                    }
                }

                check_operation(&fragment_map, op, context, &mut result);
            }
            ExecutableDefinition::FragmentDefinition(def) => {
                // Find other one with same name
                let dup = document
                    .definitions
                    .iter()
                    .take(idx)
                    .find(|other| match other {
                        ExecutableDefinition::OperationDefinition(_) => false,
                        ExecutableDefinition::FragmentDefinition(other) => {
                            other.name.name == def.name.name
                        }
                    });
                if let Some(other) = dup {
                    result.push(
                        CheckErrorMessage::DuplicateFragmentName {
                            other_position: *other.position(),
                        }
                        .with_pos(def.name.position),
                    );
                }

                check_fragment_definition(def, context, &mut result);
            }
        }
    }
    result
}

fn check_operation<'src, S: Text<'src>>(
    fragment_map: &FragmentMap<'_, 'src>,
    op: &OperationDefinition<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    let root_type = {
        let root_types = context.definitions.root_types();
        if !root_types.original_node_ref().builtin {
            // When RootTypes has a non-builtin Pos, there is an explicit schema definition.
            // This means that type for operation must also be declared explicitly.
            let root_type_name = operation_type_from_root_types(root_types, op.operation_type);
            if root_type_name.is_none() {
                result.push(
                    CheckErrorMessage::NoRootType {
                        operation_type: op.operation_type,
                    }
                    .with_pos(*op.position())
                    .with_additional_info(vec![(
                        *root_types.original_node_ref(),
                        CheckErrorMessage::RootTypesAreDefinedHere,
                    )]),
                );
                return;
            }
        }
        let root_types = root_types.unwrap_or_default();
        let root_type_name = operation_type_from_root_types(&root_types, op.operation_type);

        let Some(root_type) = context.definitions.get_type(root_type_name) else {
            result.push(
                CheckErrorMessage::UnknownType {
                    name: root_type_name.to_string(),
                }
                .with_pos(op.position),
            );
            return;
        };
        root_type
    };
    check_directives(
        context.definitions,
        op.variables_definition.as_ref(),
        &op.directives,
        match op.operation_type {
            OperationType::Query => "QUERY",
            OperationType::Mutation => "MUTATION",
            OperationType::Subscription => "SUBSCRIPTION",
        },
        result,
    );
    if let Some(ref variables_definition) = op.variables_definition {
        check_variables_definition(variables_definition, context, result);
    }
    if op.operation_type == OperationType::Subscription {
        // Single root field check
        if selection_set_has_more_than_one_fields(fragment_map, &op.selection_set) {
            result.push(
                CheckErrorMessage::SubscriptionMustHaveExactlyOneRootField.with_pos(op.position),
            );
        }
    }
    let seen_fragments = vec![];
    check_selection_set(
        fragment_map,
        &seen_fragments,
        op.variables_definition.as_ref(),
        root_type,
        &op.selection_set,
        context,
        result,
    );
}

fn operation_type_from_root_types<T>(root_types: &RootTypes<T>, op: OperationType) -> &T {
    match op {
        OperationType::Query => &root_types.query_type,
        OperationType::Mutation => &root_types.mutation_type,
        OperationType::Subscription => &root_types.subscription_type,
    }
}

fn check_fragment_definition<'src, S: Text<'src>>(
    op: &FragmentDefinition<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    let target = context.definitions.get_type(op.type_condition.name);
    let Some(target) = target else {
        result.push(
            CheckErrorMessage::UnknownType {
                name: op.type_condition.name.to_owned(),
            }
            .with_pos(op.type_condition.position),
        );
        return;
    };

    if !matches!(
        **target,
        TypeDefinition::Object(_) | TypeDefinition::Interface(_) | TypeDefinition::Union(_)
    ) {
        result.push(
            CheckErrorMessage::InvalidFragmentTarget {
                name: op.type_condition.name.to_owned(),
            }
            .with_pos(op.type_condition.position)
            .with_additional_info(vec![(
                *target.original_node_ref(),
                CheckErrorMessage::DefinitionPos {
                    name: (*target.name()).to_string(),
                },
            )]),
        );
    }
}

fn check_variables_definition<'src, S: Text<'src>>(
    variables: &VariablesDefinition<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    let mut seen_variables = vec![];
    for v in variables.definitions.iter() {
        if seen_variables.contains(&v.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedVariableName {
                    name: v.name.name.to_owned(),
                }
                .with_pos(v.pos),
            );
        } else {
            seen_variables.push(v.name.name);
        }
        let type_kind =
            inout_kind_of_type(context.definitions, v.r#type.unwrapped_type().name.name);
        match type_kind {
            None => {
                result.push(
                    CheckErrorMessage::UnknownType {
                        name: v.r#type.unwrapped_type().name.to_string(),
                    }
                    .with_pos(*v.r#type.position()),
                );
            }
            Some(t) if t.is_input_type() => {}
            _ => {
                result.push(
                    CheckErrorMessage::NoOutputType {
                        name: v.r#type.unwrapped_type().name.to_string(),
                    }
                    .with_pos(*v.r#type.position()),
                );
            }
        }
    }
}

fn check_selection_set<'src, S: Text<'src>>(
    fragment_map: &FragmentMap<'_, 'src>,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition<'src>>,
    root_type: &Node<TypeDefinition<S, Pos>, Pos>,
    selection_set: &SelectionSet<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    let root_type_name = root_type.name();
    let root_fields = direct_fields_of_output_type(&**root_type);
    let Some(root_fields) = root_fields else {
        result.push(
            CheckErrorMessage::SelectionOnInvalidType {
                kind: kind_of_type_definition(root_type),
                name: root_type_name.to_string(),
            }
            .with_pos(selection_set.position)
            .with_additional_info(vec![(
                *root_type.original_node_ref(),
                CheckErrorMessage::DefinitionPos {
                    name: root_type_name.to_string(),
                },
            )]),
        );
        return;
    };

    for selection in selection_set.selections.iter() {
        match selection {
            Selection::Field(field_selection) => {
                check_selection_field(
                    fragment_map,
                    seen_fragments,
                    variables,
                    *root_type.original_node_ref(),
                    root_type_name,
                    &root_fields,
                    field_selection,
                    context,
                    result,
                );
            }
            Selection::FragmentSpread(fragment_spread) => {
                check_fragment_spread(
                    fragment_map,
                    seen_fragments,
                    variables,
                    root_type,
                    fragment_spread,
                    context,
                    result,
                );
            }
            Selection::InlineFragment(inline_fragment) => {
                check_inline_fragment(
                    fragment_map,
                    seen_fragments,
                    variables,
                    root_type,
                    inline_fragment,
                    context,
                    result,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_selection_field<'src, S: Text<'src>, F: Borrow<Field<S, Pos>>>(
    fragment_map: &FragmentMap<'_, 'src>,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition<'src>>,
    root_type_pos: Pos,
    root_type_name: &str,
    root_fields: &[F],
    field_selection: &SelectionField<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    let selection_name = field_selection.name.name;
    let target_field = root_fields.iter().find_map(|field| {
        let f = <F as Borrow<Field<_, _>>>::borrow(field);
        (f.name == selection_name).then_some(f)
    });
    let Some(target_field) = target_field else {
        result.push(
            CheckErrorMessage::FieldNotFound {
                field_name: field_selection.name.to_string(),
                type_name: root_type_name.to_owned(),
            }
            .with_pos(*field_selection.name.position())
            .with_additional_info(vec![(
                root_type_pos,
                CheckErrorMessage::DefinitionPos {
                    name: root_type_name.to_owned(),
                },
            )]),
        );
        return;
    };

    check_directives(
        context.definitions,
        variables,
        &field_selection.directives,
        "FIELD",
        result,
    );
    check_arguments(
        context.definitions,
        variables,
        field_selection.name.position,
        field_selection.name.name,
        "field",
        field_selection.arguments.as_ref(),
        target_field.arguments.as_ref(),
        result,
    );
    let Some(target_field_type) = context
        .definitions
        .get_type(target_field.r#type.unwrapped())
    else {
        result.push(CheckErrorMessage::TypeSystemError.with_pos(field_selection.name.position));
        return;
    };

    if let Some(ref selection_set) = field_selection.selection_set {
        check_selection_set(
            fragment_map,
            seen_fragments,
            variables,
            target_field_type,
            selection_set,
            context,
            result,
        );
    } else {
        // No selection set
        if direct_fields_of_output_type(target_field_type).is_some() {
            result.push(
                CheckErrorMessage::MustSpecifySelectionSet {
                    name: field_selection.name.to_string(),
                }
                .with_pos(field_selection.name.position),
            );
        }
    }
}

fn check_fragment_spread<'src, S: Text<'src>>(
    fragment_map: &FragmentMap<'_, 'src>,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition<'src>>,
    root_type: &Node<TypeDefinition<S, Pos>, Pos>,
    fragment_spread: &FragmentSpread<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    if seen_fragments.contains(&fragment_spread.fragment_name.name) {
        result.push(
            CheckErrorMessage::RecursingFragmentSpread {
                name: fragment_spread.fragment_name.to_string(),
            }
            .with_pos(fragment_spread.position),
        );
        return;
    }
    let seen_fragments: Vec<&str> = seen_fragments
        .iter()
        .copied()
        .chain(vec![fragment_spread.fragment_name.name])
        .collect();
    let seen_fragments = &seen_fragments;
    let Some(target) = fragment_map.get(fragment_spread.fragment_name.name) else {
        result.push(
            CheckErrorMessage::UnknownFragment {
                name: fragment_spread.fragment_name.to_string(),
            }
            .with_pos(fragment_spread.fragment_name.position),
        );
        return;
    };
    let Some(fragment_condition) = context.definitions.get_type(target.type_condition.name) else {
        // This should be checked elsewhere
        return;
    };
    check_fragment_spread_core(
        fragment_map,
        seen_fragments,
        variables,
        root_type,
        fragment_spread.position,
        fragment_condition,
        &target.selection_set,
        context,
        result,
    );
}

fn check_inline_fragment<'src, S: Text<'src>>(
    fragment_map: &FragmentMap<'_, 'src>,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition<'src>>,
    root_type: &Node<TypeDefinition<S, Pos>, Pos>,
    inline_fragment: &InlineFragment<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    match inline_fragment.type_condition {
        None => {
            check_selection_set(
                fragment_map,
                seen_fragments,
                variables,
                root_type,
                &inline_fragment.selection_set,
                context,
                result,
            );
        }
        Some(ref type_cond) => {
            let Some(type_cond_definition) = context.definitions.get_type(type_cond.name) else {
                result.push(
                    CheckErrorMessage::UnknownType {
                        name: type_cond.name.to_owned(),
                    }
                    .with_pos(type_cond.position),
                );
                return;
            };
            check_fragment_spread_core(
                fragment_map,
                seen_fragments,
                variables,
                root_type,
                inline_fragment.position,
                type_cond_definition,
                &inline_fragment.selection_set,
                context,
                result,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_fragment_spread_core<'src, S: Text<'src>>(
    fragment_map: &FragmentMap<'_, 'src>,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition<'src>>,
    root_type: &Node<TypeDefinition<S, Pos>, Pos>,
    spread_pos: Pos,
    fragment_condition: &Node<TypeDefinition<S, Pos>, Pos>,
    fragment_selection_set: &SelectionSet<'src>,
    context: &OperationCheckContext<'_, 'src, S>,
    result: &mut Vec<CheckError>,
) {
    match (&**root_type, &**fragment_condition) {
        (
            TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_),
            _,
        ) => {
            // This should be flagged elsewhere
            return;
        }
        (
            TypeDefinition::Object(obj_definition),
            TypeDefinition::Object(cond_obj_definition),
        ) => {
            let cond_obj_name = cond_obj_definition.name.inner_ref();
            if obj_definition.name != *cond_obj_name {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: cond_obj_definition.name.to_string(),
                        scope: obj_definition.name.to_string(),
                    }
                    .with_pos(spread_pos)
                    .with_additional_info(vec![
                        (
                            *root_type.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: cond_obj_definition.name.to_string(),
                            },
                        ),
                        (
                            *fragment_condition.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: obj_definition.name.to_string(),
                            },
                        ),
                    ]),
                );
            }
        }
        (
            TypeDefinition::Object(obj_definition),
            TypeDefinition::Interface(intf_definition),
        )
        | (
            TypeDefinition::Interface(intf_definition),
            TypeDefinition::Object(obj_definition),
        ) => {
            let intf_name = intf_definition.name.inner_ref();
            let obj_implements_intf = obj_definition
                .interfaces
                .iter()
                .find(|im| im.inner_ref() == intf_name);
            if obj_implements_intf.is_none() {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: intf_definition.name.to_string(),
                        scope: obj_definition.name.to_string(),
                    }
                    .with_pos(spread_pos)
                    .with_additional_info(vec![
                        (
                            *root_type.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: intf_definition.name.to_string(),
                            },
                        ),
                        (
                            *fragment_condition.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: obj_definition.name.to_string(),
                            },
                        ),
                    ]),
                );
            }
        }
        (
            TypeDefinition::Object(obj_definition),
            TypeDefinition::Union(cond_union_definition),
        )
        | (
            TypeDefinition::Union(cond_union_definition),
            TypeDefinition::Object(obj_definition),
        ) => {
            let obj_name = obj_definition.name.inner_ref();
            let obj_in_union = cond_union_definition
                .possible_types
                .iter()
                .find(|mem| mem.inner_ref() == obj_name);
            if obj_in_union.is_none() {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: cond_union_definition.name.to_string(),
                        scope: obj_definition.name.to_string(),
                    }
                    .with_pos(spread_pos)
                    .with_additional_info(vec![
                        (
                            *root_type.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: cond_union_definition.name.to_string(),
                            },
                        ),
                        (
                            *fragment_condition.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: obj_definition.name.to_string(),
                            },
                        ),
                    ]),
                );
            }
        }
        (
            TypeDefinition::Interface(interface_definition1),
            TypeDefinition::Interface(interface_definition2),
        ) => {
            let intf1_name = interface_definition1.name.inner_ref();
            let intf2_name = interface_definition2.name.inner_ref();
            if intf1_name == intf2_name {
                // fast path
                return;
            }
            // When matching interfaces, we have to look for concrete types that implement both interfaces
            let any_obj_implements_both_type =
                context
                    .definitions
                    .iter_types()
                    .any(|(_, type_def)| match type_def.as_object() {
                        Some(obj_def) => {
                            obj_def
                                .interfaces
                                .iter()
                                .any(|imp| imp.inner_ref() == intf1_name)
                                && obj_def
                                    .interfaces
                                    .iter()
                                    .any(|imp| imp.inner_ref() == intf2_name)
                        }
                        None => false,
                    });
            if !any_obj_implements_both_type {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: interface_definition2.name.to_string(),
                        scope: interface_definition2.name.to_string(),
                    }
                    .with_pos(spread_pos)
                    .with_additional_info(vec![
                        (
                            *root_type.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: interface_definition2.name.to_string(),
                            },
                        ),
                        (
                            *fragment_condition.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: interface_definition1.name.to_string(),
                            },
                        ),
                    ]),
                );
            }
        }
        (
            TypeDefinition::Interface(interface_definition),
            TypeDefinition::Union(union_definition),
        )
        | (
            TypeDefinition::Union(union_definition),
            TypeDefinition::Interface(interface_definition),
        ) => {
            let intf_name = interface_definition.name.inner_ref();
            let some_member_implements_interface =
                union_definition.possible_types.iter().any(|mem| {
                    let mem_def = context
                        .definitions
                        .get_type(mem)
                        .and_then(|ty| ty.as_object());
                    match mem_def {
                        Some(mem_def) => mem_def
                            .interfaces
                            .iter()
                            .any(|imp| imp.inner_ref() == intf_name),
                        _ => {
                            result.push(
                                CheckErrorMessage::TypeSystemError
                                    .with_pos(*mem.original_node_ref()),
                            );
                            true
                        }
                    }
                });
            if !some_member_implements_interface {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: union_definition.name.to_string(),
                        scope: interface_definition.name.to_string(),
                    }
                    .with_pos(spread_pos)
                    .with_additional_info(vec![
                        (
                            *root_type.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: interface_definition.name.to_string(),
                            },
                        ),
                        (
                            *fragment_condition.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: union_definition.name.to_string(),
                            },
                        ),
                    ]),
                );
            }
        }
        (
            TypeDefinition::Union(union_definition1),
            TypeDefinition::Union(union_definition2),
        ) => {
            let there_is_overlapping_member = union_definition2.possible_types.iter().any(|mem2| {
                union_definition1
                    .possible_types
                    .iter()
                    .any(|mem1| mem1 == &**mem2)
            });
            if !there_is_overlapping_member {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: union_definition2.name.to_string(),
                        scope: union_definition1.name.to_string(),
                    }
                    .with_pos(spread_pos)
                    .with_additional_info(vec![
                        (
                            *root_type.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: union_definition1.name.to_string(),
                            },
                        ),
                        (
                            *fragment_condition.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: union_definition2.name.to_string(),
                            },
                        ),
                    ]),
                );
            }
        }
        _ => {}
    }
    check_selection_set(
        fragment_map,
        seen_fragments,
        variables,
        fragment_condition,
        fragment_selection_set,
        context,
        result,
    );
}

fn kind_of_type_definition<S, D>(definition: &TypeDefinition<S, D>) -> TypeKind {
    match definition {
        TypeDefinition::Scalar(_) => TypeKind::Scalar,
        TypeDefinition::Object(_) => TypeKind::Object,
        TypeDefinition::Interface(_) => TypeKind::Interface,
        TypeDefinition::Union(_) => TypeKind::Union,
        TypeDefinition::Enum(_) => TypeKind::Enum,
        TypeDefinition::InputObject(_) => TypeKind::InputObject,
    }
}
