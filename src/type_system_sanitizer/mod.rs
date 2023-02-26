use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::graphql_parser::ast::{
    base::{HasPos, Ident, Pos},
    directive::Directive,
    type_system::{
        ArgumentsDefinition, DirectiveDefinition, EnumTypeDefinition, InterfaceTypeDefinition,
        ObjectTypeDefinition, ScalarTypeDefinition, SchemaDefinition, TypeDefinition,
        TypeSystemDefinition, UnionTypeDefinition,
    },
    TypeSystemDocument,
};

use self::{
    builtins::generate_builtins, check_directive_recursion::check_directive_recursion,
    definition_map::DefinitionMap, interfaces::check_valid_implementation, types::kind_of_type,
};

mod builtins;
mod check_directive_recursion;
mod definition_map;
mod interfaces;
mod tests;
mod types;

/// Checks for invalid type system definition document.
pub fn check_type_system_document(document: &TypeSystemDocument) -> Vec<CheckTypeSystemError> {
    let mut definition_map = generate_definition_map(document);
    let (builtin_types, builtin_directives) = generate_builtins();
    definition_map
        .types
        .extend(builtin_types.iter().map(|(key, def)| (*key, def)));
    definition_map
        .directives
        .extend(builtin_directives.iter().map(|(key, def)| (*key, def)));

    let definition_map = definition_map;

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
                _ => {}
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

#[derive(Error, Debug)]
pub enum CheckTypeSystemError {
    #[error("Name that starts with '__' is reserved")]
    UnscoUnsco { position: Pos },
    #[error("Name '{name}' is duplicated")]
    DuplicatedName { position: Pos, name: String },
    #[error("Directive name '{name}' is not found")]
    UnknownDirective { position: Pos, name: String },
    #[error("Type '{name}' is not found")]
    UnknownType { position: Pos, name: String },
    #[error("Directive '{name}' is not allowed for this location")]
    DirectiveLocationNotAllowed { position: Pos, name: String },
    #[error("Repeated application of directive '{name}' is not allowed")]
    RepeatedDirective { position: Pos, name: String },
    #[error("Directive '{name}' is recursing")]
    RecursingDirective { position: Pos, name: String },
    #[error("Output type '{name}' is not allowed here")]
    NoOutputType { position: Pos, name: String },
    #[error("Input type '{name}' is not allowed here")]
    NoInputType { position: Pos, name: String },
    #[error("'{name}' is not an interface")]
    NotInterface { position: Pos, name: String },
    #[error("This type must implement interface '{name}'")]
    InterfaceNotImplemented { position: Pos, name: String },
    #[error("Interface must not implement itself")]
    NoImplementSelf { position: Pos },
    #[error("This type must have a field '{field_name}' from interface '{interface_name}'")]
    InterfaceFieldNotImplemented {
        position: Pos,
        field_name: String,
        interface_name: String,
    },
    #[error(
        "Type of this argument does not match the same argument from interface '{interface_name}'"
    )]
    FieldTypeMisMatchWithInterface {
        position: Pos,
        interface_name: String,
    },
    #[error("Type of this filed does not match the same field from interface '{interface_name}'")]
    InterfaceArgumentNotImplemented {
        position: Pos,
        argument_name: String,
        interface_name: String,
    },
    #[error(
        "Type of this argument does not match the same argument from interface '{interface_name}'"
    )]
    ArgumentTypeMisMatchWithInterface {
        position: Pos,
        interface_name: String,
    },
    #[error(
        "Type of this argument must be nullable because it is not in the same field from interface '{interface_name}'"
    )]
    ArgumentTypeNonNullAgainstInterface {
        position: Pos,
        interface_name: String,
    },
    #[error("'{member_name}' is not an object type")]
    NonObjectTypeUnionMember { position: Pos, member_name: String },
}

fn generate_definition_map<'a>(document: &'a TypeSystemDocument<'a>) -> DefinitionMap<'a> {
    let mut result = DefinitionMap::new();
    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(schema) => {
                result.schema.insert(schema);
            }
            TypeSystemDefinition::TypeDefinition(def) => {
                result.types.insert(
                    def.name().expect("Type definition should always have name"),
                    def,
                );
            }
            TypeSystemDefinition::DirectiveDefinition(def) => {
                result.directives.insert(def.name.name, def);
            }
        }
    }

    result
}

fn check_schema(
    d: &SchemaDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    check_directives(definitions, &d.directives, "SCHEMA", result);
}

fn check_directive<'a>(
    d: &DirectiveDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    check_directive_recursion(definitions, d, result);

    if name_starts_with_unscounsco(&d.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *d.name.position(),
        });
    }
    if let Some(ref arg) = d.arguments {
        check_arguments_definition(arg, definitions, result);
    }
}

fn check_scalar(
    scalar: &ScalarTypeDefinition,
    definition_map: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&scalar.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *scalar.name.position(),
        })
    }
    check_directives(definition_map, &scalar.directives, "SCALAR", result);
}

fn check_object(
    object: &ObjectTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&object.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *object.name.position(),
        })
    }
    check_directives(definitions, &object.directives, "OBJECT", result);

    let mut seen_fields = vec![];
    for f in object.fields.iter() {
        if seen_fields.contains(&f.name.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: *f.name.position(),
                name: f.name.name.to_owned(),
            });
        } else {
            seen_fields.push(f.name.name);
        }
        if name_starts_with_unscounsco(&f.name) {
            result.push(CheckTypeSystemError::UnscoUnsco {
                position: *f.name.position(),
            })
        }
        if kind_of_type(definitions, &f.r#type).map_or(false, |k| !k.is_output_type()) {
            result.push(CheckTypeSystemError::NoInputType {
                position: *f.r#type.position(),
                name: f.r#type.unwrapped_type().name.name.to_owned(),
            });
        }
        if let Some(ref arg) = f.arguments {
            check_arguments_definition(arg, definitions, result)
        }
    }
    for interface in object.implements.iter() {
        let Some(interface_def) = definitions.types.get(interface.name) else {
            result.push(CheckTypeSystemError::UnknownType { position: *interface.position(), name: interface.name.to_owned() });
            continue;
        };
        let TypeDefinition::Interface(ref def) = interface_def else {
            result.push(CheckTypeSystemError::NotInterface  { position: *interface.position(), name: interface.name.to_owned() });
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
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&interface.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *interface.name.position(),
        })
    }
    check_directives(definitions, &interface.directives, "INTERFACE", result);

    let mut seen_fields = vec![];
    for f in interface.fields.iter() {
        if seen_fields.contains(&f.name.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: *f.name.position(),
                name: f.name.name.to_owned(),
            });
        } else {
            seen_fields.push(f.name.name);
        }
        if name_starts_with_unscounsco(&f.name) {
            result.push(CheckTypeSystemError::UnscoUnsco {
                position: *f.name.position(),
            })
        }
        if kind_of_type(definitions, &f.r#type).map_or(false, |k| !k.is_output_type()) {
            result.push(CheckTypeSystemError::NoInputType {
                position: *f.r#type.position(),
                name: f.r#type.unwrapped_type().name.name.to_owned(),
            });
        }
        if let Some(ref arg) = f.arguments {
            check_arguments_definition(arg, definitions, result)
        }
    }
    for other_interface in interface.implements.iter() {
        if interface.name.name == other_interface.name {
            result.push(CheckTypeSystemError::NoImplementSelf {
                position: other_interface.position,
            });
            continue;
        }
        let Some(interface_def) = definitions.types.get(other_interface.name) else {
            result.push(CheckTypeSystemError::UnknownType { position: *other_interface.position(), name: other_interface.name.to_owned() });
            continue;
        };
        let TypeDefinition::Interface(ref def) = interface_def else {
            result.push(CheckTypeSystemError::NotInterface  { position: *other_interface.position(), name: other_interface.name.to_owned() });
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
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&union.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *union.name.position(),
        })
    }
    check_directives(definitions, &union.directives, "UNION", result);

    let mut seen_members = vec![];
    for member in union.members.iter() {
        if seen_members.contains(&member.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: member.position,
                name: member.name.to_owned(),
            })
        } else {
            seen_members.push(member.name);
        }
        // The member types of a Union type must all be Object base types;
        let member_type_def = definitions.types.get(member.name);
        match member_type_def {
            None => {
                result.push(CheckTypeSystemError::UnknownType {
                    position: member.position,
                    name: member.name.to_owned(),
                });
            }
            Some(member_type_def) => {
                if !matches!(member_type_def, TypeDefinition::Object(_)) {
                    result.push(CheckTypeSystemError::NonObjectTypeUnionMember {
                        position: member.position,
                        member_name: member.name.to_owned(),
                    });
                }
            }
        }
    }
}

fn check_enum(
    enum_def: &EnumTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&enum_def.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *enum_def.name.position(),
        })
    }
    check_directives(definitions, &enum_def.directives, "ENUM", result);

    let mut seen_values = vec![];
    for v in enum_def.values.iter() {
        if seen_values.contains(&v.name.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: v.name.position,
                name: v.name.name.to_owned(),
            })
        } else {
            seen_values.push(v.name.name);
        }
        check_directives(definitions, &v.directives, "ENUM_VALUE", result)
    }
}

fn check_arguments_definition(
    def: &ArgumentsDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    let mut argument_names = vec![];
    for v in def.input_values.iter() {
        if name_starts_with_unscounsco(&v.name) {
            result.push(CheckTypeSystemError::UnscoUnsco {
                position: *v.name.position(),
            });
        }
        if argument_names.contains(&v.name.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: *v.name.position(),
                name: v.name.name.to_owned(),
            })
        } else {
            argument_names.push(v.name.name);
        }
        let type_is_not_input_type =
            kind_of_type(definitions, &v.r#type).map_or(false, |k| !k.is_input_type());
        if type_is_not_input_type {
            result.push(CheckTypeSystemError::NoOutputType {
                position: *v.r#type.position(),
                name: v.r#type.unwrapped_type().name.name.to_owned(),
            })
        }

        check_directives(definitions, &v.directives, "ARGUMENT_DEFINITION", result)
    }
}

fn name_starts_with_unscounsco(name: &Ident) -> bool {
    name.name.starts_with("__")
}

fn check_directives(
    definitions: &DefinitionMap,
    directives: &[Directive],
    current_position: &str,
    result: &mut Vec<CheckTypeSystemError>,
) {
    let mut seen_directives = vec![];
    for d in directives {
        match definitions.directives.get(d.name.name) {
            None => result.push(CheckTypeSystemError::UnknownDirective {
                position: *d.name.position(),
                name: d.name.name.to_owned(),
            }),
            Some(def) => {
                if def
                    .locations
                    .iter()
                    .find(|loc| loc.name == current_position)
                    .is_none()
                {
                    result.push(CheckTypeSystemError::DirectiveLocationNotAllowed {
                        position: *d.position(),
                        name: d.name.name.to_owned(),
                    });
                }
                if seen_directives.contains(&d.name.name) {
                    if def.repeatable.is_none() {
                        result.push(CheckTypeSystemError::RepeatedDirective {
                            position: *d.position(),
                            name: d.name.name.to_owned(),
                        })
                    }
                } else {
                    seen_directives.push(d.name.name);
                }
            }
        }
    }
}
