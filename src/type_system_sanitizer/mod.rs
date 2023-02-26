use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::graphql_parser::ast::{
    base::{HasPos, Ident, Pos},
    directive::Directive,
    type_system::{
        DirectiveDefinition, ScalarTypeDefinition, SchemaDefinition, TypeDefinition,
        TypeSystemDefinition,
    },
    TypeSystemDocument,
};

use self::{
    check_directive_recursion::check_directive_recursion, definition_map::DefinitionMap,
    types::kind_of_type,
};

mod check_directive_recursion;
mod definition_map;
mod tests;
mod types;

/// Checks for invalid type system definition document.
pub fn check_type_system_document(document: &TypeSystemDocument) -> Vec<CheckTypeSystemError> {
    let definition_map = generate_definition_map(document);
    let mut result = vec![];

    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(ref d) => {
                check_schema(d, &definition_map, &mut result);
            }
            TypeSystemDefinition::DirectiveDefinition(ref d) => {
                check_directive(d, &definition_map, &mut result);
            }
            _ => {}
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
    #[error("Directive '{name}' is not allowed for this location")]
    DirectiveLocationNotAllowed { position: Pos, name: String },
    #[error("Repeated application of directive '{name}' is not allowed")]
    RepeatedDirective { position: Pos, name: String },
    #[error("Directive '{name}' is recursing")]
    RecursingDirective { position: Pos, name: String },
    #[error("Output type '{name}' is not allowed here")]
    NoOutputType { position: Pos, name: String },
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
        let mut argument_names = HashSet::new();
        for v in arg.input_values.iter() {
            if name_starts_with_unscounsco(&v.name) {
                result.push(CheckTypeSystemError::UnscoUnsco {
                    position: *v.name.position(),
                });
            }
            let name_is_new = argument_names.insert(&v.name.name);
            if !name_is_new {
                result.push(CheckTypeSystemError::DuplicatedName {
                    position: *v.name.position(),
                    name: v.name.name.to_owned(),
                })
            }
            let type_is_not_input_type =
                kind_of_type(definitions, &v.r#type).map_or(false, |k| !k.is_input_type());
            if type_is_not_input_type {
                result.push(CheckTypeSystemError::NoOutputType {
                    position: *v.r#type.position(),
                    name: v.r#type.unwrapped_type().name.name.to_owned(),
                })
            }
        }
    }
}

fn validate_scalars(
    scalars: &[&ScalarTypeDefinition],
    definition_map: &DefinitionMap,
) -> Vec<CheckTypeSystemError> {
    let mut result = vec![];

    for scalar in scalars {
        if name_starts_with_unscounsco(&scalar.name) {
            result.push(CheckTypeSystemError::UnscoUnsco {
                position: *scalar.name.position(),
            })
        }
        check_directives(definition_map, &scalar.directives, "SCALAR", &mut result);
    }

    result
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
