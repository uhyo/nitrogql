use crate::ast::{
    base::{HasPos, Ident},
    type_system::{
        ArgumentsDefinition, DirectiveDefinition, EnumTypeDefinition, InputObjectTypeDefinition,
        InterfaceTypeDefinition, ObjectTypeDefinition, ScalarTypeDefinition, SchemaDefinition,
        TypeDefinition, TypeSystemDefinition, UnionTypeDefinition,
    },
    TypeSystemDocument,
};

use self::{
    check_directive_recursion::check_directive_recursion, interfaces::check_valid_implementation,
};

use super::{
    common::check_directives,
    definition_map::{generate_definition_map, DefinitionMap},
    error::{CheckError, CheckErrorMessage},
    types::inout_kind_of_type,
};

mod check_directive_recursion;
mod interfaces;
#[cfg(test)]
mod tests;

/// Checks for invalid type system definition document.
pub fn check_type_system_document(document: &TypeSystemDocument) -> Vec<CheckError> {
    let definition_map = generate_definition_map(document);

    let mut result = vec![];

    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(ref d) => {
                check_schema(d, &definition_map, &mut result);
            }
            TypeSystemDefinition::TypeDefinition(ref d) => match d {
                TypeDefinition::Scalar(ref d) => {
                    check_scalar(d, &definition_map, &mut result);
                }
                TypeDefinition::Object(ref d) => {
                    check_object(d, &definition_map, &mut result);
                }
                TypeDefinition::Interface(ref d) => {
                    check_interface(d, &definition_map, &mut result);
                }
                TypeDefinition::Union(ref d) => {
                    check_union(d, &definition_map, &mut result);
                }
                TypeDefinition::Enum(ref d) => {
                    check_enum(d, &definition_map, &mut result);
                }
                TypeDefinition::InputObject(ref d) => {
                    check_input_object(d, &definition_map, &mut result);
                }
            },
            TypeSystemDefinition::DirectiveDefinition(ref d) => {
                check_directive(d, &definition_map, &mut result);
            }
        }
    }

    // result.append(&mut validate_scalars(
    //     &scalar_definitions[..],
    //     &directive_by_name,
    // ));

    result
}

fn check_schema(d: &SchemaDefinition, definitions: &DefinitionMap, result: &mut Vec<CheckError>) {
    check_directives(definitions, None, &d.directives, "SCHEMA", result);
}

fn check_directive<'a>(
    d: &DirectiveDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    check_directive_recursion(definitions, d, result);

    if name_starts_with_unscounsco(&d.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(*d.name.position()));
    }
    if let Some(ref arg) = d.arguments {
        check_arguments_definition(arg, definitions, result);
    }
}

fn check_scalar(
    scalar: &ScalarTypeDefinition,
    definition_map: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    if name_starts_with_unscounsco(&scalar.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(scalar.name.position))
    }
    check_directives(definition_map, None, &scalar.directives, "SCALAR", result);
}

fn check_object(
    object: &ObjectTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    if name_starts_with_unscounsco(&object.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(*object.name.position()));
    }
    check_directives(definitions, None, &object.directives, "OBJECT", result);

    let mut seen_fields = vec![];
    for f in object.fields.iter() {
        if seen_fields.contains(&f.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedName {
                    name: f.name.name.to_owned(),
                }
                .with_pos(*f.name.position()),
            );
        } else {
            seen_fields.push(f.name.name);
        }
        if name_starts_with_unscounsco(&f.name) {
            result.push(CheckErrorMessage::UnscoUnsco.with_pos(*f.name.position()));
        }
        match inout_kind_of_type(definitions, &f.r#type).map(|k| k.is_output_type()) {
            Some(true) => {}
            Some(false) => {
                result.push(
                    CheckErrorMessage::NoInputType {
                        name: f.r#type.unwrapped_type().name.name.to_owned(),
                    }
                    .with_pos(*f.r#type.position()),
                );
            }
            None => {
                result.push(
                    CheckErrorMessage::UnknownType {
                        name: f.r#type.unwrapped_type().name.name.to_owned(),
                    }
                    .with_pos(*f.r#type.position()),
                );
            }
        }
        if let Some(ref arg) = f.arguments {
            check_arguments_definition(arg, definitions, result)
        }
    }
    for interface in object.implements.iter() {
        let Some(interface_def) = definitions.types.get(interface.name) else {
            result.push(CheckErrorMessage::UnknownType { name: interface.name.to_owned() }.with_pos(*interface.position()));
            continue;
        };
        let TypeDefinition::Interface(ref def) = interface_def else {
            result.push(CheckErrorMessage::NotInterface  { name: interface.name.to_owned() }.with_pos(*interface.position()));
            continue;
        };
        check_valid_implementation(
            definitions,
            &object.name,
            &object.fields,
            &object.implements,
            def,
            result,
        );
    }
}

fn check_interface(
    interface: &InterfaceTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    if name_starts_with_unscounsco(&interface.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(*interface.name.position()));
    }
    check_directives(
        definitions,
        None,
        &interface.directives,
        "INTERFACE",
        result,
    );

    let mut seen_fields = vec![];
    for f in interface.fields.iter() {
        if seen_fields.contains(&f.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedName {
                    name: f.name.name.to_owned(),
                }
                .with_pos(*f.name.position()),
            );
        } else {
            seen_fields.push(f.name.name);
        }
        if name_starts_with_unscounsco(&f.name) {
            result.push(CheckErrorMessage::UnscoUnsco.with_pos(*f.name.position()));
        }
        if inout_kind_of_type(definitions, &f.r#type).map_or(false, |k| !k.is_output_type()) {
            result.push(
                CheckErrorMessage::NoInputType {
                    name: f.r#type.unwrapped_type().name.name.to_owned(),
                }
                .with_pos(*f.r#type.position()),
            );
        }
        if let Some(ref arg) = f.arguments {
            check_arguments_definition(arg, definitions, result)
        }
    }
    for other_interface in interface.implements.iter() {
        if interface.name.name == other_interface.name {
            result.push(CheckErrorMessage::NoImplementSelf.with_pos(other_interface.position));
            continue;
        }
        let Some(interface_def) = definitions.types.get(other_interface.name) else {
            result.push(CheckErrorMessage::UnknownType { name: other_interface.name.to_owned() }.with_pos(*other_interface.position()));
            continue;
        };
        let TypeDefinition::Interface(ref def) = interface_def else {
            result.push(CheckErrorMessage::NotInterface  { name: other_interface.name.to_owned() }
            .with_pos(*other_interface.position()));
            continue;
        };
        check_valid_implementation(
            definitions,
            &interface.name,
            &interface.fields,
            &interface.implements,
            def,
            result,
        );
    }
}

fn check_union(
    union: &UnionTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    if name_starts_with_unscounsco(&union.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(*union.name.position()));
    }
    check_directives(definitions, None, &union.directives, "UNION", result);

    let mut seen_members = vec![];
    for member in union.members.iter() {
        if seen_members.contains(&member.name) {
            result.push(
                CheckErrorMessage::DuplicatedName {
                    name: member.name.to_owned(),
                }
                .with_pos(member.position),
            );
        } else {
            seen_members.push(member.name);
        }
        // The member types of a Union type must all be Object base types;
        let member_type_def = definitions.types.get(member.name);
        match member_type_def {
            None => {
                result.push(
                    CheckErrorMessage::UnknownType {
                        name: member.name.to_owned(),
                    }
                    .with_pos(member.position),
                );
            }
            Some(member_type_def) => {
                if !matches!(member_type_def, TypeDefinition::Object(_)) {
                    result.push(
                        CheckErrorMessage::NonObjectTypeUnionMember {
                            member_name: member.name.to_owned(),
                        }
                        .with_pos(member.position),
                    );
                }
            }
        }
    }
}

fn check_enum(
    enum_def: &EnumTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    if name_starts_with_unscounsco(&enum_def.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(*enum_def.name.position()));
    }
    check_directives(definitions, None, &enum_def.directives, "ENUM", result);

    let mut seen_values = vec![];
    for v in enum_def.values.iter() {
        if seen_values.contains(&v.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedName {
                    name: v.name.name.to_owned(),
                }
                .with_pos(v.name.position),
            );
        } else {
            seen_values.push(v.name.name);
        }
        check_directives(definitions, None, &v.directives, "ENUM_VALUE", result)
    }
}

fn check_input_object(
    input: &InputObjectTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    if name_starts_with_unscounsco(&input.name) {
        result.push(CheckErrorMessage::UnscoUnsco.with_pos(*input.name.position()));
    }
    check_directives(definitions, None, &input.directives, "INPUT_OBJECT", result);

    let mut seen_fields = vec![];
    for f in input.fields.iter() {
        if seen_fields.contains(&f.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedName {
                    name: f.name.name.to_owned(),
                }
                .with_pos(f.name.position),
            )
        } else {
            seen_fields.push(f.name.name);
        }
        if name_starts_with_unscounsco(&f.name) {
            result.push(CheckErrorMessage::UnscoUnsco.with_pos(f.name.position));
        }
        check_directives(
            definitions,
            None,
            &f.directives,
            "INPUT_FIELD_DEFINITION",
            result,
        );

        let type_is_not_input_type =
            inout_kind_of_type(definitions, &f.r#type).map(|k| !k.is_input_type());
        match type_is_not_input_type {
            None => {
                result.push(
                    CheckErrorMessage::UnknownType {
                        name: f.r#type.unwrapped_type().name.name.to_owned(),
                    }
                    .with_pos(*f.r#type.position()),
                );
            }
            Some(true) => {
                result.push(
                    CheckErrorMessage::NoOutputType {
                        name: f.r#type.unwrapped_type().name.name.to_owned(),
                    }
                    .with_pos(*f.r#type.position()),
                );
            }
            Some(false) => {}
        }
    }
}

fn check_arguments_definition(
    def: &ArgumentsDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckError>,
) {
    let mut argument_names = vec![];
    for v in def.input_values.iter() {
        if name_starts_with_unscounsco(&v.name) {
            result.push(CheckErrorMessage::UnscoUnsco.with_pos(*v.name.position()));
        }
        if argument_names.contains(&v.name.name) {
            result.push(
                CheckErrorMessage::DuplicatedName {
                    name: v.name.name.to_owned(),
                }
                .with_pos(v.name.position),
            );
        } else {
            argument_names.push(v.name.name);
        }
        let type_is_not_input_type =
            inout_kind_of_type(definitions, &v.r#type).map_or(false, |k| !k.is_input_type());
        if type_is_not_input_type {
            result.push(
                CheckErrorMessage::NoOutputType {
                    name: v.r#type.unwrapped_type().name.name.to_owned(),
                }
                .with_pos(*v.r#type.position()),
            );
        }

        check_directives(
            definitions,
            None,
            &v.directives,
            "ARGUMENT_DEFINITION",
            result,
        )
    }
}

fn name_starts_with_unscounsco(name: &Ident) -> bool {
    name.name.starts_with("__")
}
