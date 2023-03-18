use std::collections::HashSet;

use crate::{
    definition_map::DefinitionMap,
    error::{CheckError, CheckErrorMessage},
};
use nitrogql_ast::{
    directive::Directive,
    type_system::{DirectiveDefinition, TypeDefinition},
};

/// Checks and generates diagnostics for recursed directives.
pub fn check_directive_recursion(
    definition_map: &DefinitionMap,
    directive: &DirectiveDefinition,
    result: &mut Vec<CheckError>,
) {
    let mut seen_directives = HashSet::new();
    let mut current_directives = vec![directive];
    loop {
        let mut next_directives: Vec<&DirectiveDefinition> = vec![];
        for d in current_directives.into_iter() {
            if seen_directives.contains(d.name.name) {
                // Recursion!
                result.push(
                    CheckErrorMessage::RecursingDirective {
                        name: d.name.name.to_owned(),
                    }
                    .with_pos(d.position),
                );
                continue;
            }
            seen_directives.insert(d.name.name);
            next_directives.extend(
                d.arguments
                    .iter()
                    .flat_map(|arguments| arguments.input_values.iter())
                    .flat_map(|def| {
                        let type_definition = definition_map
                            .types
                            .get(def.r#type.unwrapped_type().name.name);
                        let type_definition_directives = type_definition
                            .into_iter()
                            .flat_map(|def| directives_in_type(def));

                        def.directives.iter().chain(type_definition_directives)
                    })
                    .flat_map(|directive| {
                        definition_map
                            .directives
                            .get(directive.name.name)
                            .into_iter()
                    })
                    .map(|ptr| *ptr),
            );
        }
        if next_directives.is_empty() {
            break;
        }
        current_directives = next_directives;
    }
}

fn directives_in_type<'a>(def: &'a TypeDefinition<'a>) -> Vec<&'a Directive<'a>> {
    match def {
        TypeDefinition::Scalar(def) => def.directives.iter().collect(),
        TypeDefinition::Object(def) => def
            .directives
            .iter()
            .chain(def.fields.iter().flat_map(|f| f.directives.iter()))
            .collect(),
        TypeDefinition::Interface(def) => def
            .directives
            .iter()
            .chain(def.fields.iter().flat_map(|f| f.directives.iter()))
            .collect(),
        TypeDefinition::Union(def) => def.directives.iter().collect(),
        TypeDefinition::Enum(def) => def
            .directives
            .iter()
            .chain(def.values.iter().flat_map(|v| v.directives.iter()))
            .collect(),
        TypeDefinition::InputObject(def) => def
            .directives
            .iter()
            .chain(def.fields.iter().flat_map(|f| f.directives.iter()))
            .collect(),
    }
}
