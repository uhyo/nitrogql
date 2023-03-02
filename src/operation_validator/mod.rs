use crate::{
    graphql_parser::ast::{
        base::HasPos,
        directive::Directive,
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
            VariablesDefinition,
        },
        selection_set::{SelectionSet, Selection},
        type_system::{FieldDefinition, TypeDefinition},
        OperationDocument, TypeSystemDocument,
    },
    type_system_validator::{generate_definition_map, DefinitionMap},
};

use self::error::{CheckOperationError, CheckOperationErrorMessage, TypeKind};

mod error;

pub fn check_operation_document(
    schema: &TypeSystemDocument,
    document: &OperationDocument,
) -> Vec<CheckOperationError> {
    let mut result = vec![];
    let definitions = generate_definition_map(schema);

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
                                CheckOperationErrorMessage::UnNamedOperationMustBeSingle
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
                                CheckOperationErrorMessage::DuplicateOperationName {
                                    operation_type: op.operation_type,
                                    other_position: *other.position(),
                                }
                                .with_pos(name.position),
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
                        CheckOperationErrorMessage::DuplicateFragmentName {
                            other_position: *other.position(),
                        }
                        .with_pos(def.name.position),
                    );
                }

                check_fragment(&definitions, def, &mut result);
            }
        }
    }
    vec![]
}

fn check_operation(
    definitions: &DefinitionMap,
    op: &OperationDefinition,
    result: &mut Vec<CheckOperationError>,
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
                        CheckOperationErrorMessage::TypeNotFound { name: root_type_name.to_owned() }.with_pos(op.position)
                    );
                    return;
                };
                root_type
            }
            Err(schema_def) => {
                result.push(
                    CheckOperationErrorMessage::NoRootType {
                        operation_type: op.operation_type,
                        schema_definition: schema_def.position,
                    }
                    .with_pos(*op.position()),
                );
                return;
            }
        }
    };
    check_directives(definitions, &op.directives, result);
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

fn check_fragment(
    definitions: &DefinitionMap,
    op: &FragmentDefinition,
    result: &mut Vec<CheckOperationError>,
) {
    todo!()
}

fn check_directives(
    definitions: &DefinitionMap,
    directives: &[Directive],
    result: &mut Vec<CheckOperationError>,
) {
}

fn check_variables_definition(
    definitions: &DefinitionMap,
    variables: &VariablesDefinition,
    result: &mut Vec<CheckOperationError>,
) {
}

fn check_selection_set(
    definitions: &DefinitionMap,
    variables: Option<&VariablesDefinition>,
    root_type: &TypeDefinition,
    selection_set: &SelectionSet,
    result: &mut Vec<CheckOperationError>,
) {
    let root_type_name = root_type.name().expect("Type definition must have name");
    let root_fields = direct_fields_of_output_type(definitions, root_type);
    let Some(root_fields) = root_fields else {
        result.push(
            CheckOperationErrorMessage::SelectionOnInvalidType { kind: 
                kind_of_type_definition(root_type),
                name: root_type_name.to_owned(),
                type_def: *root_type.position(),
            }
                .with_pos(selection_set.position)
        );
        return;
    };

    for selection in selection_set.selections.iter() {
        match selection {
            Selection::Field(field_selection) => {
                let target_field = root_fields.iter().find(|field| {
                    field.name.name == field_selection.name.name
                });
                let Some(target_field) = target_field else {
                    result.push(
                        CheckOperationErrorMessage::FieldNotFound { field_name: 
                            field_selection.name.name.to_owned(),
                             type_name: root_type_name.to_owned(), type_def: *root_type.position(),
                         }.with_pos(field_selection.name.position)
                    );
                    continue;
                };
            }
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!(),
        }
    }
}

fn direct_fields_of_output_type<'a, 'src>(
    definitions: &'a DefinitionMap,
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
