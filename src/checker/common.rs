use log::warn;

use crate::graphql_parser::ast::{
    base::{HasPos, Ident, Pos, Variable},
    directive::Directive,
    operations::{VariableDefinition, VariablesDefinition},
    r#type::{NamedType, Type},
    type_system::{ArgumentsDefinition, TypeDefinition},
    value::{Arguments, Value},
};

use super::{
    definition_map::DefinitionMap,
    error::{CheckError, CheckErrorMessage},
};

pub fn check_directives(
    definitions: &DefinitionMap,
    directives: &[Directive],
    current_position: &str,
    result: &mut Vec<CheckError>,
) {
    let mut seen_directives = vec![];
    for d in directives {
        match definitions.directives.get(d.name.name) {
            None => result.push(
                CheckErrorMessage::UnknownDirective {
                    name: d.name.name.to_owned(),
                }
                .with_pos(d.name.position),
            ),
            Some(def) => {
                if def
                    .locations
                    .iter()
                    .find(|loc| loc.name == current_position)
                    .is_none()
                {
                    result.push(
                        CheckErrorMessage::DirectiveLocationNotAllowed {
                            name: d.name.name.to_owned(),
                        }
                        .with_pos(d.position),
                    );
                }
                if seen_directives.contains(&d.name.name) {
                    if def.repeatable.is_none() {
                        result.push(
                            CheckErrorMessage::RepeatedDirective {
                                name: d.name.name.to_owned(),
                            }
                            .with_pos(d.position),
                        )
                    }
                } else {
                    seen_directives.push(d.name.name);
                }

                match (&d.arguments, &def.arguments) {
                    (None, None) => {}
                    (Some(ref args), None) => {
                        result.push(
                            CheckErrorMessage::ArgumentsNotNeeded { kind: "directive" }
                                .with_pos(args.position)
                                .with_additional_info(vec![(
                                    def.position,
                                    CheckErrorMessage::DefinitionPos {
                                        name: def.name.name.to_owned(),
                                    },
                                )]),
                        );
                    }
                    (args, Some(ref args_def)) => {
                        check_arguments(
                            definitions,
                            None,
                            d.position,
                            args.as_ref(),
                            args_def,
                            result,
                        );
                    }
                }
            }
        }
    }
}

pub fn check_arguments(
    definitions: &DefinitionMap,
    variables: Option<&VariablesDefinition>,
    parent_pos: Pos,
    arguments: Option<&Arguments>,
    arguments_definition: &ArgumentsDefinition,
    result: &mut Vec<CheckError>,
) {
    let argument_pos = arguments.map_or(parent_pos, |args| args.position);
    let arguments: Vec<_> = arguments
        .into_iter()
        .flat_map(|arg| arg.arguments.iter())
        .collect();
    let mut seen_args = 0;
    for arg_def in arguments_definition.input_values.iter() {
        let arg = arguments
            .iter()
            .find(|(arg_name, _)| arg_name.name == arg_def.name.name);
        match arg {
            None => {
                let null_is_allowed = 'b: {
                    if !arg_def.r#type.is_nonnull() {
                        break 'b true;
                    }
                    match arg_def.default_value {
                        None => false,
                        Some(ref v) if matches!(v, Value::NullValue(_)) => false,
                        Some(_) => true,
                    }
                };
                if null_is_allowed {
                    seen_args += 1;
                } else {
                    result.push(
                        CheckErrorMessage::RequiredArgumentNotSpecified {
                            name: arg_def.name.name.to_owned(),
                        }
                        .with_pos(argument_pos)
                        .with_additional_info(vec![(
                            arg_def.position,
                            CheckErrorMessage::DefinitionPos {
                                name: arg_def.name.name.to_owned(),
                            },
                        )]),
                    )
                }
            }
            Some((_, arg_value)) => {
                check_value(definitions, variables, arg_value, &arg_def.r#type, result);
                seen_args += 1;
            }
        }
    }
    if seen_args < arguments.len() {
        // There are extra arguments
        for (arg_name, _) in arguments {
            if arguments_definition
                .input_values
                .iter()
                .find(|arg_def| arg_def.name.name == arg_name.name)
                .is_none()
            {
                result.push(
                    CheckErrorMessage::UnknownArgument {
                        name: arg_name.name.to_owned(),
                    }
                    .with_pos(arg_name.position),
                );
            }
        }
    }
}

pub fn check_value(
    definitions: &DefinitionMap,
    variables: Option<&VariablesDefinition>,
    value: &Value,
    expected_type: &Type,
    result: &mut Vec<CheckError>,
) {
    let mut additional_info = vec![];
    let is_mismatch = 'b: {
        if let Value::Variable(variable) = value {
            let Some(v_def) = get_variable_definition(variables, variable) else {
                result.push(
                    CheckErrorMessage::UnknownVariable { name: variable.name.to_owned() }
                    .with_pos(*value.position())
                );
                return;
            };
            break 'b !check_type_compatibility(definitions, &v_def.r#type, expected_type);
        }
        match expected_type {
            Type::NonNull(inner) => match value {
                Value::NullValue(v) => true,
                Value::Variable(_) => unreachable!(),
                value => {
                    check_value(definitions, variables, value, &inner.r#type, result);
                    false
                }
            },
            Type::List(expected_inner) => match value {
                Value::ListValue(inner) => {
                    for elem in inner.values.iter() {
                        check_value(definitions, variables, elem, &expected_inner.r#type, result);
                    }
                    false
                }
                Value::Variable(_) => unreachable!(),
                _ => true,
            },
            Type::Named(expected_name) => {
                let Some(type_def) = definitions.types.get(expected_name.name.name) else {
                    // unknown type name
                    result.push(
                        CheckErrorMessage::TypeSystemError
                        .with_pos(*expected_name.name.position())
                    );
                    return;
                };
                let (is_compatible, a) =
                    is_value_compatible_type_def(definitions, variables, value, type_def, result);
                additional_info.extend(a);
                !is_compatible
            }
        }
    };
    if is_mismatch {
        result.push(
            CheckErrorMessage::TypeMismatch {
                r#type: expected_type.to_string(),
            }
            .with_pos(*value.position())
            .with_additional_info(additional_info),
        );
    }
}

// Note: this function does not consider Value::Variable
fn is_value_compatible_type_def(
    definitions: &DefinitionMap,
    variables: Option<&VariablesDefinition>,
    value: &Value,
    expected_type: &TypeDefinition,
    result: &mut Vec<CheckError>,
) -> (bool, Vec<(Pos, CheckErrorMessage)>) {
    match expected_type {
        TypeDefinition::Scalar(scalar_def) => {
            // TODO: better handling of scalar, including custom scalars
            (
                match scalar_def.name.name {
                    "Boolean" => matches!(value, Value::BooleanValue(_)),
                    "Int" => matches!(value, Value::IntValue(_)),
                    "Float" => matches!(value, Value::FloatValue(_)),
                    "String" => matches!(value, Value::StringValue(_)),
                    "ID" => matches!(value, Value::StringValue(_)),
                    custom_scalar => {
                        warn!(
                            "Not checking value compatibility for custom scalar '{}'",
                            custom_scalar
                        );
                        true
                    }
                },
                vec![],
            )
        }
        TypeDefinition::Object(_) | TypeDefinition::Interface(_) | TypeDefinition::Union(_) => {
            // These are never inputs
            (false, vec![])
        }
        TypeDefinition::Enum(enum_def) => match value {
            Value::EnumValue(value) => {
                if enum_def
                    .values
                    .iter()
                    .find(|v| v.name.name == value.value)
                    .is_none()
                {
                    result.push(
                        CheckErrorMessage::UnknownEnumMember {
                            member: value.value.to_owned(),
                            r#enum: enum_def.name.name.to_owned(),
                        }
                        .with_pos(value.position)
                        .with_additional_info(vec![(
                            enum_def.position,
                            CheckErrorMessage::DefinitionPos {
                                name: enum_def.name.name.to_owned(),
                            },
                        )]),
                    );
                }
                (true, vec![])
            }
            _ => (false, vec![]),
        },
        TypeDefinition::InputObject(object_def) => {
            let Value::ObjectValue(value) = value else {
                return (false, vec![]);
            };
            let mut res = true;
            let mut additional_info = vec![];
            let mut seen_fields = 0;
            for expected_field in object_def.fields.iter() {
                let value_field = value
                    .fields
                    .iter()
                    .find(|(key, _)| key.name == expected_field.name.name);
                match value_field {
                    None => {
                        if expected_field.r#type.is_nonnull()
                            && expected_field.default_value.is_none()
                        {
                            // When field does not exist and the expected field is both non-nullable and has no default value, then it is an error.
                            res = false;
                            additional_info.push((
                                expected_field.position,
                                CheckErrorMessage::RequiredFieldNotSpecified {
                                    name: expected_field.name.name.to_owned(),
                                },
                            ));
                        } else {
                            seen_fields += 1;
                        }
                    }
                    Some((_, value)) => {
                        check_value(
                            definitions,
                            variables,
                            value,
                            &expected_field.r#type,
                            result,
                        );
                        seen_fields += 1;
                    }
                }
            }
            if seen_fields < value.fields.len() {
                // Value has extraneous field
                res = false;
                for (key, _) in value.fields.iter() {
                    let field_def = object_def.fields.iter().find(|f| f.name.name == key.name);
                    if field_def.is_none() {
                        additional_info.push((
                            key.position,
                            CheckErrorMessage::UnknownField {
                                name: key.name.to_owned(),
                            },
                        ))
                    }
                }
            }
            (res, additional_info)
        }
    }
}

/// Returns true if `value_type` is assignable to `expected_type`.
fn check_type_compatibility(
    definitions: &DefinitionMap,
    value_type: &Type,
    expected_type: &Type,
) -> bool {
    // https://spec.graphql.org/draft/#AreTypesCompatible()
    match (expected_type, value_type) {
        (Type::NonNull(expected_inner), Type::NonNull(value_inner)) => {
            check_type_compatibility(definitions, &value_inner.r#type, &expected_inner.r#type)
        }
        (_, Type::NonNull(value_inner)) => {
            check_type_compatibility(definitions, &value_inner.r#type, expected_type)
        }
        (Type::NonNull(_), _) => false,
        (Type::List(expected_inner), Type::List(value_inner)) => {
            check_type_compatibility(definitions, &value_inner.r#type, &expected_inner.r#type)
        }
        (Type::List(_), _) => false,
        (_, Type::List(_)) => false,
        (Type::Named(expected_name), Type::Named(value_inner)) => {
            expected_name.name.name == value_inner.name.name
        }
    }
}

fn builtin_type(name: &str) -> Type {
    Type::Named(NamedType {
        name: Ident {
            name,
            position: Pos::builtin(),
        },
    })
}

fn get_variable_definition<'a, 'src>(
    variables: Option<&'a VariablesDefinition<'src>>,
    variable: &'a Variable<'src>,
) -> Option<&'a VariableDefinition<'src>> {
    variables.and_then(|variables| {
        variables
            .definitions
            .iter()
            .find(|def| def.name.name == variable.name)
    })
}
