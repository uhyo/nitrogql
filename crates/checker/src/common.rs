use graphql_type_system::{InputValue, OriginalNodeRef, Schema, Type, TypeDefinition};
use log::warn;

use nitrogql_ast::{
    base::{HasPos, Pos},
    directive::Directive,
    value::{Arguments, Value},
    variable::{Variable, VariableDefinition, VariablesDefinition},
};
use nitrogql_semantics::type_system_utils::convert_type;

use super::error::{CheckError, CheckErrorMessage};

pub fn check_directives(
    definitions: &Schema<&str, Pos>,
    variables: Option<&VariablesDefinition>,
    directives: &[Directive],
    current_position: &str,
    result: &mut Vec<CheckError>,
) {
    let mut seen_directives = vec![];
    for d in directives {
        match definitions.get_directive(d.name.name) {
            None => result.push(
                CheckErrorMessage::UnknownDirective {
                    name: d.name.to_string(),
                }
                .with_pos(d.name.position),
            ),
            Some(def) => {
                if def
                    .locations
                    .iter()
                    .find(|loc| **loc == current_position)
                    .is_none()
                {
                    result.push(
                        CheckErrorMessage::DirectiveLocationNotAllowed {
                            name: d.name.to_string(),
                        }
                        .with_pos(d.position),
                    );
                }
                if seen_directives.contains(&d.name.name) {
                    if def.repeatable.is_none() {
                        result.push(
                            CheckErrorMessage::RepeatedDirective {
                                name: d.name.to_string(),
                            }
                            .with_pos(d.position),
                        )
                    }
                } else {
                    seen_directives.push(d.name.name);
                }

                check_arguments(
                    definitions,
                    variables,
                    d.position,
                    d.name.name,
                    "directive",
                    d.arguments.as_ref(),
                    def.arguments.as_ref(),
                    result,
                );
            }
        }
    }
}

pub fn check_arguments(
    definitions: &Schema<&str, Pos>,
    variables: Option<&VariablesDefinition>,
    parent_pos: Pos,
    parent_name: &str,
    parent_kind: &'static str,
    arguments: Option<&Arguments>,
    arguments_definition: &[InputValue<&str, Pos>],
    result: &mut Vec<CheckError>,
) {
    match arguments {
        None if arguments_definition.is_empty() => {}
        Some(args) if arguments_definition.is_empty() => {
            result.push(
                CheckErrorMessage::ArgumentsNotNeeded { kind: parent_kind }
                    .with_pos(args.position)
                    .with_additional_info(vec![(
                        parent_pos,
                        CheckErrorMessage::DefinitionPos {
                            name: parent_name.to_owned(),
                        },
                    )]),
            );
        }
        arguments => {
            let argument_pos = arguments.map_or(parent_pos, |args| args.position);
            let arguments: Vec<_> = arguments
                .into_iter()
                .flat_map(|arg| arg.arguments.iter())
                .collect();
            let mut seen_args = 0;
            for arg_def in arguments_definition.iter() {
                let arg = arguments
                    .iter()
                    .find(|(arg_name, _)| arg_def.name == arg_name.name);
                match arg {
                    None => {
                        let null_is_allowed = 'b: {
                            if !arg_def.r#type.is_nonnull() {
                                break 'b true;
                            }
                            match arg_def.default_value {
                                None => false,
                                // TODO: maybe check for null default value
                                // Some(ref v) if matches!(v, Value::NullValue(_)) => false,
                                Some(_) => true,
                            }
                        };
                        if !null_is_allowed {
                            result.push(
                                CheckErrorMessage::RequiredArgumentNotSpecified {
                                    name: arg_def.name.to_string(),
                                }
                                .with_pos(argument_pos)
                                .with_additional_info(vec![(
                                    *arg_def.name.original_node_ref(),
                                    CheckErrorMessage::DefinitionPos {
                                        name: arg_def.name.to_string(),
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
                        .iter()
                        .find(|arg_def| arg_def.name == arg_name.name)
                        .is_none()
                    {
                        result.push(
                            CheckErrorMessage::UnknownArgument {
                                name: arg_name.to_string(),
                            }
                            .with_pos(arg_name.position),
                        );
                    }
                }
            }
        }
    }
}

pub fn check_value(
    definitions: &Schema<&str, Pos>,
    variables: Option<&VariablesDefinition>,
    value: &Value,
    expected_type: &Type<&str, Pos>,
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
            break 'b !check_type_compatibility(
                definitions,
                &convert_type(&v_def.r#type),
                expected_type,
            );
        }
        match expected_type {
            Type::NonNull(inner) => match value {
                Value::NullValue(_) => true,
                Value::Variable(_) => unreachable!(),
                value => {
                    check_value(definitions, variables, value, inner, result);
                    false
                }
            },
            Type::List(expected_inner) => match value {
                Value::ListValue(inner) => {
                    for elem in inner.values.iter() {
                        check_value(definitions, variables, elem, expected_inner, result);
                    }
                    false
                }
                Value::Variable(_) => unreachable!(),
                _ => true,
            },
            Type::Named(expected_name) => {
                let Some(type_def) = definitions.get_type(expected_name) else {
                    // unknown type name
                    result.push(
                        CheckErrorMessage::TypeSystemError
                        .with_pos(*expected_name.original_node_ref())
                        .with_additional_info(vec![(
                            *expected_name.original_node_ref(),
                            CheckErrorMessage::UnknownType { name: expected_name.to_string() }
                        )])
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
    definitions: &Schema<&str, Pos>,
    variables: Option<&VariablesDefinition>,
    value: &Value,
    expected_type: &TypeDefinition<&str, Pos>,
    result: &mut Vec<CheckError>,
) -> (bool, Vec<(Pos, CheckErrorMessage)>) {
    match expected_type {
        TypeDefinition::Scalar(scalar_def) => {
            // TODO: better handling of scalar, including custom scalars
            (
                match *scalar_def.name {
                    "Boolean" => matches!(value, Value::BooleanValue(_) | Value::NullValue(_)),
                    "Int" => matches!(value, Value::IntValue(_) | Value::NullValue(_)),
                    "Float" => matches!(value, Value::FloatValue(_) | Value::NullValue(_)),
                    "String" => matches!(value, Value::StringValue(_) | Value::NullValue(_)),
                    "ID" => matches!(value, Value::StringValue(_) | Value::NullValue(_)),
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
            Value::NullValue(_) => (true, vec![]),
            Value::EnumValue(value) => {
                if enum_def
                    .members
                    .iter()
                    .find(|v| v.name == value.value)
                    .is_none()
                {
                    result.push(
                        CheckErrorMessage::UnknownEnumMember {
                            member: value.value.to_owned(),
                            r#enum: enum_def.name.to_string(),
                        }
                        .with_pos(value.position)
                        .with_additional_info(vec![(
                            *enum_def.name.original_node_ref(),
                            CheckErrorMessage::DefinitionPos {
                                name: enum_def.name.to_string(),
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
                if matches!(value, Value::NullValue(_)) {
                    return (true, vec![]);
                }
                return (false, vec![]);
            };
            let mut res = true;
            let mut additional_info = vec![];
            let mut seen_fields = 0;
            for expected_field in object_def.fields.iter() {
                let value_field = value
                    .fields
                    .iter()
                    .find(|(key, _)| expected_field.name == key.name);
                match value_field {
                    None => {
                        if expected_field.r#type.is_nonnull()
                            && expected_field.default_value.is_none()
                        {
                            // When field does not exist and the expected field is both non-nullable and has no default value, then it is an error.
                            res = false;
                            additional_info.push((
                                *expected_field.original_node_ref(),
                                CheckErrorMessage::RequiredFieldNotSpecified {
                                    name: expected_field.name.to_string(),
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
                    let field_def = object_def.fields.iter().find(|f| f.name == key.name);
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
    definitions: &Schema<&str, Pos>,
    value_type: &Type<&str, Pos>,
    expected_type: &Type<&str, Pos>,
) -> bool {
    // https://spec.graphql.org/draft/#AreTypesCompatible()
    match (expected_type, value_type) {
        (Type::NonNull(expected_inner), Type::NonNull(value_inner)) => {
            check_type_compatibility(definitions, value_inner, expected_inner)
        }
        (_, Type::NonNull(value_inner)) => {
            check_type_compatibility(definitions, value_inner, expected_type)
        }
        (Type::NonNull(_), _) => false,
        (Type::List(expected_inner), Type::List(value_inner)) => {
            check_type_compatibility(definitions, value_inner, expected_inner)
        }
        (Type::List(_), _) => false,
        (_, Type::List(_)) => false,
        (Type::Named(expected_name), Type::Named(value_inner)) => **expected_name == **value_inner,
    }
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
