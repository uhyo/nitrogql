use crate::{
    graphql_parser::ast::{
        base::HasPos,
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
            VariablesDefinition,
        },
        selection_set::{SelectionSet, Selection},
        type_system::{FieldDefinition, TypeDefinition},
        OperationDocument, TypeSystemDocument,
    },
};

use super::{definition_map::{DefinitionMap, generate_definition_map}, error::{CheckError, CheckErrorMessage, TypeKind}, common::{check_directives, check_arguments}, types::{inout_kind_of_type, TypeInOutKind}, builtins::generate_builtins};

#[cfg(test)]
mod tests;

pub fn check_operation_document(
    schema: &TypeSystemDocument,
    document: &OperationDocument,
) -> Vec<CheckError> {
    let mut result = vec![];
    let mut definitions = generate_definition_map(schema);
    let (builtin_types, builtin_directives) = generate_builtins();
    definitions
        .types
        .extend(builtin_types.iter().map(|(key, def)| (*key, def)));
    definitions
        .directives
        .extend(builtin_directives.iter().map(|(key, def)| (*key, def)));

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

                check_operation(&definitions, op, &mut result);
            }
            ExecutableDefinition::FragmentDefinition(def) => {
                // Find other one with same name
                let dup = document
                    .definitions
                    .iter()
                    .take(idx)
                    .find(|other| match other {
                        ExecutableDefinition::OperationDefinition(def) => false,
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
        todo!("Single root field check");
    }
    check_selection_set(
        definitions,
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
                (*target.position(), CheckErrorMessage::DefinitionPos { name: target.name().expect("Type must have a name").to_owned() })
            ])
        );
    }
    // todo: fragment must be used somewhere in document
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
    variables: Option<&VariablesDefinition>,
    root_type: &TypeDefinition,
    selection_set: &SelectionSet,
    result: &mut Vec<CheckError>,
) {
    let root_type_name = root_type.name().expect("Type definition must have name");
    let root_fields = direct_fields_of_output_type(root_type);
    let Some(root_fields) = root_fields else {
        result.push(
            CheckErrorMessage::SelectionOnInvalidType { kind: 
                kind_of_type_definition(root_type),
                name: root_type_name.to_owned(),
            }
                .with_pos(selection_set.position)
                .with_additional_info(vec![
                    (*root_type.position(), CheckErrorMessage::DefinitionPos { name: root_type_name.to_owned()})
                ])
        );
        return;
    };

    let mut seen_selected_names = vec![];

    for selection in selection_set.selections.iter() {
        match selection {
            Selection::Field(field_selection) => {
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
                            (*root_type.position(), CheckErrorMessage::DefinitionPos {
                                name: root_type_name.to_owned()
                             })
                         ])
                    );
                    continue;
                };
                let selected_name = field_selection.alias.map_or(field_selection.name.name, |a| a.name);
                if seen_selected_names.contains(&selected_name) {
                    result.push(
                        CheckErrorMessage::DuplicateSelectionName { name: selected_name.to_owned() }
                        .with_pos(field_selection.alias.map_or(field_selection.name.position, |a| a.position))
                    );
                } else {
                    seen_selected_names.push(selected_name);
                }

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
                        continue;
                    };

                    check_selection_set(definitions, variables, target_field_type, selection_set, result);
                }
            }
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!(),
        }
    }
}

fn direct_fields_of_output_type<'a, 'src>(
    ty: &'a TypeDefinition<'src>,
) -> Option<&'a [FieldDefinition<'src>]> {
    match ty {
        TypeDefinition::Object(obj) => Some(&obj.fields),
        TypeDefinition::Interface(obj) => Some(&obj.fields),
        TypeDefinition::Union(_)
        | TypeDefinition::Scalar(_)
        | TypeDefinition::Enum(_)
        | TypeDefinition::InputObject(_) => None,
    }
}

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
