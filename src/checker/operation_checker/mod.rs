use once_cell::sync::Lazy;

use crate::{
    ast::{
        base::{HasPos, Pos, Ident},
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
            VariablesDefinition,
        },
        selection_set::{SelectionSet, Selection, Field, FragmentSpread, InlineFragment},
        type_system::{FieldDefinition, TypeDefinition},
        OperationDocument, TypeSystemDocument, r#type::{Type, NonNullType, NamedType},
    },
};

use self::{fragment_map::{generate_fragment_map, FragmentMap}, count_selection_set_fields::selection_set_has_more_than_one_fields};

use super::{definition_map::{DefinitionMap, generate_definition_map}, error::{CheckError, CheckErrorMessage, TypeKind}, common::{check_directives, check_arguments}, types::{inout_kind_of_type}};

#[cfg(test)]
mod tests;
mod fragment_map;
mod count_selection_set_fields;

pub fn check_operation_document(
    schema: &TypeSystemDocument,
    document: &OperationDocument,
) -> Vec<CheckError> {
    let mut result = vec![];
    let definitions = generate_definition_map(schema);

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
                                    def.name.map_or(false, |n| n.name == name.name)
                                }
                                ExecutableDefinition::FragmentDefinition(_) => false,
                            });
                        if let Some(other) = dup {
                            result.push(
                                CheckErrorMessage::DuplicateOperationName {
                                    operation_type: op.operation_type,
                                }
                                .with_pos(name.position).with_additional_info(vec![(
                                    *other.position(),
                                    CheckErrorMessage::AnotherDefinitionPos { name: name.name.to_owned()  }
                                )]),
                            );
                        }
                    }
                }

                check_operation(&definitions, &fragment_map, op, &mut result);
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

                check_fragment_definition(&definitions, def, &mut result);
            }
        }
    }
    result
}

fn check_operation(
    definitions: &DefinitionMap,
    fragment_map: &FragmentMap,
    op: &OperationDefinition,
    result: &mut Vec<CheckError>,
) {
    let root_type = {
        let root_type_name = definitions
            .schema
            .map(|schema_def| {
                schema_def
                    .definitions
                    .iter()
                    .find_map(|(key, value)| (*key == op.operation_type).then_some(value.name))
                    .ok_or(schema_def)
            })
            .unwrap_or_else(|| {
                Ok(match op.operation_type {
                    OperationType::Query => "Query",
                    OperationType::Mutation => "Mutation",
                    OperationType::Subscription => "Subscription",
                })
            });

        match root_type_name {
            Ok(root_type_name) => {
                let Some(root_type) = definitions.types.get(root_type_name) else {
                    result.push(
                        CheckErrorMessage::UnknownType { name: root_type_name.to_owned() }.with_pos(op.position)
                    );
                    return;
                };
                root_type
            }
            Err(schema_def) => {
                result.push(
                    CheckErrorMessage::NoRootType {
                        operation_type: op.operation_type,
                    }
                    .with_pos(*op.position())
                    .with_additional_info(vec![
                        (schema_def.position, CheckErrorMessage::RootTypesAreDefinedHere)
                    ])
                );
                return;
            }
        }
    };
    check_directives(definitions,
        op.variables_definition.as_ref(),
         &op.directives, match op.operation_type {
        OperationType::Query => "QUERY",
        OperationType::Mutation => "MUTATION",
        OperationType::Subscription => "SUBSCRIPTION",
    }, result);
    if let Some(ref variables_definition) = op.variables_definition {
        check_variables_definition(definitions, variables_definition, result);
    }
    if op.operation_type == OperationType::Subscription {
        // Single root field check
        if selection_set_has_more_than_one_fields(fragment_map, &op.selection_set) {
            result.push(
                CheckErrorMessage::SubscriptionMustHaveExactlyOneRootField
                .with_pos(op.position)
            );
        }
    }
    let seen_fragments = vec![];
    check_selection_set(
        definitions,
        fragment_map,
        &seen_fragments,
        op.variables_definition.as_ref(),
        root_type,
        &op.selection_set,
        result,
    );
}

fn check_fragment_definition(
    definitions: &DefinitionMap,
    op: &FragmentDefinition,
    result: &mut Vec<CheckError>,
) {
    let target = definitions.types.get(op.type_condition.name);
    let Some(target) = target else {
        result.push(
            CheckErrorMessage::UnknownType { name: op.type_condition.name.to_owned() }
            .with_pos(op.type_condition.position)
        );
        return;
    };

    if !matches!(
        target,
        TypeDefinition::Object(_) | TypeDefinition::Interface(_) | TypeDefinition::Union(_)
    ) {
        result.push(
            CheckErrorMessage::InvalidFragmentTarget { name: op.type_condition.name.to_owned() }
            .with_pos(op.type_condition.position)
            .with_additional_info(vec![
                (*target.position(), CheckErrorMessage::DefinitionPos { name: target.name().name.to_owned() })
            ])
        );
        return;
    }

}

fn check_variables_definition(
    definitions: &DefinitionMap,
    variables: &VariablesDefinition,
    result: &mut Vec<CheckError>,
) {
    let mut seen_variables = vec![];
    for v in variables.definitions.iter() {
        if seen_variables.contains(&v.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedVariableName { name: v.name.name.to_owned() }
                .with_pos(v.pos)
            );
        } else {
            seen_variables.push(v.name.name);
        }
        let type_kind = inout_kind_of_type(definitions, &v.r#type);
        match type_kind {
            None => {
                result.push(
                    CheckErrorMessage::UnknownType { name: v.r#type.unwrapped_type().name.name.to_owned() }
                    .with_pos(*v.r#type.position())
                );
            }
            Some(t) if t.is_input_type() => {
            }
            _ => {
                result.push(
                    CheckErrorMessage::NoOutputType { name: v.r#type.unwrapped_type().name.name.to_owned() }
                    .with_pos(*v.r#type.position())
                );
            }
        }
    }
}

fn check_selection_set(
    definitions: &DefinitionMap,
    fragment_map: &FragmentMap,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition>,
    root_type: &TypeDefinition,
    selection_set: &SelectionSet,
    result: &mut Vec<CheckError>,
) {
    let root_type_name = root_type.name();
    let root_fields = direct_fields_of_output_type(root_type);
    let Some(root_fields) = root_fields else {
        result.push(
            CheckErrorMessage::SelectionOnInvalidType { kind: 
                kind_of_type_definition(root_type),
                name: root_type_name.name.to_owned(),
            }
                .with_pos(selection_set.position)
                .with_additional_info(vec![
                    (*root_type.position(), CheckErrorMessage::DefinitionPos { name: root_type_name.name.to_owned()})
                ])
        );
        return;
    };

    for selection in selection_set.selections.iter() {
        match selection {
            Selection::Field(field_selection) => {
                check_selection_field(
                    definitions,
                    fragment_map,
                    seen_fragments,
                    variables,
                    *root_type.position(),
                    root_type_name.name,
                    &root_fields,
                    field_selection,
                    result,
                );
                
            }
            Selection::FragmentSpread(fragment_spread) => {
                check_fragment_spread(definitions, fragment_map, seen_fragments, variables, root_type, fragment_spread, result);
            },
            Selection::InlineFragment(inline_fragment) => {
                check_inline_fragment(definitions, fragment_map, seen_fragments, variables, root_type, inline_fragment, result);
            }
        }
    }
}

fn check_selection_field(
    definitions: &DefinitionMap,
    fragment_map: &FragmentMap,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition>,
    root_type_pos: Pos,
    root_type_name: &str,
    root_fields: &[&FieldDefinition],
    field_selection: &Field,
    result: &mut Vec<CheckError>
) {
    let target_field = root_fields.iter().find(|field| {
        field.name.name == field_selection.name.name
    });
    let Some(target_field) = target_field else {
        result.push(
            CheckErrorMessage::FieldNotFound { field_name: 
                field_selection.name.name.to_owned(),
                    type_name: root_type_name.to_owned(),
                }.with_pos(field_selection.name.position)
                .with_additional_info(vec![
                (root_type_pos, CheckErrorMessage::DefinitionPos {
                    name: root_type_name.to_owned()
                    })
                ])
        );
        return;
    };

    check_directives(definitions, variables, &field_selection.directives, "FIELD", result);
    check_arguments(
        definitions,
        variables,
        field_selection.name.position,
        field_selection.name.name,
        "field",
        field_selection.arguments.as_ref(),
        target_field.arguments.as_ref(),
        result,
    );
    if let Some(ref selection_set) = field_selection.selection_set {
        let Some(target_field_type) = definitions.types.get(
            target_field.r#type.unwrapped_type().name.name
        ) else {
            result.push(CheckErrorMessage::TypeSystemError.with_pos(selection_set.position));
            return;
        };

        check_selection_set(definitions, fragment_map, seen_fragments, variables, target_field_type, selection_set, result);
    }

}

fn check_fragment_spread(
    definitions: &DefinitionMap,
    fragment_map: &FragmentMap,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition>,
    root_type: &TypeDefinition,
    fragment_spread: &FragmentSpread,
    result: &mut Vec<CheckError>
) {
    
    if seen_fragments.contains(&fragment_spread.fragment_name.name) {
        result.push(
            CheckErrorMessage::RecursingFragmentSpread { name: fragment_spread.fragment_name.name.to_owned() }
            .with_pos(fragment_spread.position)
        );
        return;
    }
    let seen_fragments: Vec<&str> = seen_fragments.iter().map(|s| *s).chain(vec![fragment_spread.fragment_name.name]).collect();
    let seen_fragments = &seen_fragments;
    let Some(target) = fragment_map.get(fragment_spread.fragment_name.name) else {
        result.push(
            CheckErrorMessage::UnknownFragment { name: fragment_spread.fragment_name.name.to_owned() }
            .with_pos(fragment_spread.fragment_name.position)
        );
        return;
    };
    let Some(fragment_condition) = definitions.types.get(target.type_condition.name) else {
        // This should be checked elsewhere
        return;
    };
    check_fragment_spread_core(
        definitions,
        fragment_map,
        seen_fragments,
        variables,
        root_type,
        fragment_spread.position,
        fragment_condition,
        &target.selection_set,
        result,
    );
}

fn check_inline_fragment(
    definitions: &DefinitionMap,
    fragment_map: &FragmentMap,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition>,
    root_type: &TypeDefinition,
    inline_fragment: &InlineFragment,
    result: &mut Vec<CheckError>
) {
    match inline_fragment.type_condition {
        None => {
            check_selection_set(definitions, fragment_map, seen_fragments, variables, root_type, &inline_fragment.selection_set, result);
        }
        Some(ref type_cond) => {
            let Some(type_cond_definition) = definitions.types.get(type_cond.name) else {
                result.push(
                    CheckErrorMessage::UnknownType { name: type_cond.name.to_owned() }
                    .with_pos(type_cond.position)
                );
                return;
            };
        check_fragment_spread_core(
            definitions,
            fragment_map,
            seen_fragments,
            variables,
            root_type,
            inline_fragment.position,
            type_cond_definition,
            &inline_fragment.selection_set,
            result,
        );
        }
    }
}

fn check_fragment_spread_core(
    definitions: &DefinitionMap,
    fragment_map: &FragmentMap,
    seen_fragments: &[&str],
    variables: Option<&VariablesDefinition>,
    root_type: &TypeDefinition,
    spread_pos: Pos,
    fragment_condition: &TypeDefinition,
    fragment_selection_set: &SelectionSet,
    result: &mut Vec<CheckError>
) {
    match (root_type, fragment_condition) {
        (TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_), _) => {
            // This should be flagged elsewhere
            return
        }
        (TypeDefinition::Object(obj_definition), TypeDefinition::Object(cond_obj_definition)) => {
            if obj_definition.name.name != cond_obj_definition.name.name {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches { condition: cond_obj_definition.name.name.to_owned(), scope: 
                        obj_definition.name.name.to_owned()
                        }
                        .with_pos(spread_pos)
                        .with_additional_info(vec![
                        (
                            cond_obj_definition.position,
                            CheckErrorMessage::DefinitionPos { name: cond_obj_definition.name.name.to_owned() }
                        ),
                        (
                            obj_definition.position,
                            CheckErrorMessage::DefinitionPos { name: obj_definition.name.name.to_owned() }
                        ),
                        ])
                );
            }
        }
        (TypeDefinition::Object(obj_definition), TypeDefinition::Interface(intf_definition)) |
        (TypeDefinition::Interface(intf_definition), TypeDefinition::Object(obj_definition)) => {
            let obj_implements_intf = obj_definition.implements.iter().find(|im| im.name == intf_definition.name.name);
            if obj_implements_intf.is_none() {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches { condition: intf_definition.name.name.to_owned(), scope: 
                        obj_definition.name.name.to_owned()
                        }
                        .with_pos(spread_pos)
                        .with_additional_info(vec![
                        (
                            intf_definition.position,
                            CheckErrorMessage::DefinitionPos { name: intf_definition.name.name.to_owned() }
                        ),
                        (
                            obj_definition.position,
                            CheckErrorMessage::DefinitionPos { name: obj_definition.name.name.to_owned() }
                        ),
                        ])
                );
            }
        }
        (TypeDefinition::Object(obj_definition), TypeDefinition::Union(cond_union_definition)) |
        (TypeDefinition::Union(cond_union_definition), TypeDefinition::Object(obj_definition)) => {
            let obj_in_union = cond_union_definition.members.iter().find(|mem| mem.name == obj_definition.name.name);
            if obj_in_union.is_none() {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches { condition: cond_union_definition.name.name.to_owned(), scope: 
                        obj_definition.name.name.to_owned()
                        }
                        .with_pos(spread_pos)
                        .with_additional_info(vec![
                        (
                            cond_union_definition.position,
                            CheckErrorMessage::DefinitionPos { name: cond_union_definition.name.name.to_owned() }
                        ),
                        (
                            obj_definition.position,
                            CheckErrorMessage::DefinitionPos { name: obj_definition.name.name.to_owned() }
                        ),
                        ])
                );
            }
        }
        (TypeDefinition::Interface(interface_definition1), TypeDefinition::Interface(interface_definition2)) => {
            if interface_definition1.name.name == interface_definition2.name.name {
                // fast path
                return
            }
            // When matching interfaces, we have to look for concrete types that implement both interfaces 
            let any_obj_implements_both_type = definitions.types.iter().any(|(_, type_def)| {
                match type_def {
                    TypeDefinition::Object(obj_def) => {
                        obj_def.implements.iter().any(|imp| imp.name == interface_definition1.name.name) &&
                        obj_def.implements.iter().any(|imp| imp.name == interface_definition2.name.name)
                    }
                    _ => false
                }
            });
            if !any_obj_implements_both_type {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: interface_definition2.name.name.to_owned(),
                        scope: interface_definition2.name.name.to_owned(),
                    }
                        .with_pos(spread_pos)
                        .with_additional_info(vec![
                        (
                            interface_definition2.position,
                            CheckErrorMessage::DefinitionPos { name: interface_definition2.name.name.to_owned() }
                        ),
                        (
                            interface_definition1.position,
                            CheckErrorMessage::DefinitionPos { name: interface_definition1.name.name.to_owned() }
                        ),
                    ])
                );
            }
        }
        (TypeDefinition::Interface(interface_definition), TypeDefinition::Union(union_definition)) |
        (TypeDefinition::Union(union_definition), TypeDefinition::Interface(interface_definition)) => {
            let some_member_implements_interface = union_definition.members.iter().any(|mem| {
                let mem_def = definitions.types.get(mem.name);
                match mem_def {
                    Some(TypeDefinition::Object(mem_def)) => {
                        mem_def.implements.iter().any(|imp| {
                            imp.name == interface_definition.name.name
                        })
                    }
                    _ => {
                        result.push(
                            CheckErrorMessage::TypeSystemError.with_pos(mem.position)
                        );
                        true
                    }
                }
            });
            if !some_member_implements_interface {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: union_definition.name.name.to_owned(),
                        scope: interface_definition.name.name.to_owned(),
                    }
                        .with_pos(spread_pos)
                        .with_additional_info(vec![
                        (
                            interface_definition.position,
                            CheckErrorMessage::DefinitionPos { name: interface_definition.name.name.to_owned() }
                        ),
                        (
                            union_definition.position,
                            CheckErrorMessage::DefinitionPos { name: union_definition.name.name.to_owned() }
                        ),
                    ])
                );
            }
        }
        (TypeDefinition::Union(union_definition1), TypeDefinition::Union(union_definition2)) => {
            let there_is_overlapping_member = union_definition2.members.iter().any(|mem2| {
                union_definition1.members.iter().any(|mem1| mem1.name == mem2.name)
            });
            if !there_is_overlapping_member {
                result.push(
                    CheckErrorMessage::FragmentConditionNeverMatches {
                        condition: union_definition2.name.name.to_owned(),
                        scope: union_definition1.name.name.to_owned(),
                    }
                        .with_pos(spread_pos)
                        .with_additional_info(vec![
                        (
                            union_definition2.position,
                            CheckErrorMessage::DefinitionPos { name: union_definition1.name.name.to_owned() }
                        ),
                        (
                            union_definition1.position,
                            CheckErrorMessage::DefinitionPos { name: union_definition2.name.name.to_owned() }
                        ),
                    ])
                );
            }
        }
        _ => {}
    }
    check_selection_set(definitions, fragment_map, seen_fragments, variables, fragment_condition, fragment_selection_set, result);
}

pub fn direct_fields_of_output_type<'a, 'src>(
    ty: &'a TypeDefinition<'src>,
) -> Option<Vec<&'a FieldDefinition<'src>>> {
    let meta_field : &FieldDefinition = &TYPENAME_META_FIELD;
    match ty {
        TypeDefinition::Object(obj) => Some(obj.fields.iter().chain(vec![meta_field]).collect()),
        TypeDefinition::Interface(obj) => Some(obj.fields.iter().chain(vec![meta_field]).collect()),
        TypeDefinition::Union(_) => Some(vec![meta_field]),
        | TypeDefinition::Scalar(_)
        | TypeDefinition::Enum(_)
        | TypeDefinition::InputObject(_) => None,
    }
}

static TYPENAME_META_FIELD: Lazy<FieldDefinition<'static>> = Lazy::new(|| FieldDefinition {
    description: None,
    name: Ident { name: "__typename", position: Pos::builtin() },
    arguments: None,
    r#type: Type::NonNull(Box::new(NonNullType {
        r#type: Type::Named(NamedType {
            name: Ident { name: "String", position: Pos::builtin() },
        })
    })),
    directives: vec![],
});

fn kind_of_type_definition(definition: &TypeDefinition) -> TypeKind {
    match definition {
        TypeDefinition::Scalar(_) => TypeKind::Scalar,
        TypeDefinition::Object(_) => TypeKind::Object,
        TypeDefinition::Interface(_) => TypeKind::Interface,
        TypeDefinition::Union(_) => TypeKind::Union,
        TypeDefinition::Enum(_) => TypeKind::Enum,
        TypeDefinition::InputObject(_) => TypeKind::InputObject,
    }
}
