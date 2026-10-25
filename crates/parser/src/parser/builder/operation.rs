use crate::parts;
use nitrogql_ast::{
    operation::{FragmentDefinition, OperationDefinition, OperationType},
    operation_ext::{ExecutableDefinitionExt, ImportDefinition, ImportTarget},
    variable::{VariableDefinition, VariablesDefinition},
};

use super::{
    Rule,
    base::build_variable,
    directives::build_directives,
    selection_set::build_selection_set,
    r#type::build_type,
    type_system::build_description,
    utils::PairExt,
    value::{build_string_value, build_value},
};
use pest::iterators::Pair;

/// Parses a VariablesDefinition Pair.
pub fn build_variables_definition(pair: Pair<Rule>, file: usize) -> VariablesDefinition {
    let position = pair.to_pos(file);
    let defs = pair.all_children(Rule::VariableDefinition);
    VariablesDefinition {
        position,
        definitions: defs
            .into_iter()
            .map(|pair| build_variable_definition(pair, file))
            .collect(),
    }
}

/// Parses one VariableDefinition Pair.
pub fn build_variable_definition(pair: Pair<Rule>, file: usize) -> VariableDefinition {
    let pos = pair.to_pos(file);
    let (description, variable, ty, default_value, directives) = parts!(
        pair,
        Description opt,
        Variable,
        Type,
        DefaultValue opt,
        Directives opt
    );

    VariableDefinition {
        pos,
        description: description.map(|pair| build_description(pair, file)),
        name: build_variable(variable, file),
        r#type: build_type(ty, file),
        default_value: default_value.map(|pair| {
            let child = pair.only_child();
            build_value(child, file)
        }),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
    }
}

pub fn build_executable_definition(pair: Pair<Rule>, file: usize) -> ExecutableDefinitionExt {
    let pair = pair.only_child();
    let position = pair.to_pos(file);
    match pair.as_rule() {
        Rule::OperationDefinition => {
            // The grammar allows two forms for an OperationDefinition:
            //   1. Description? OperationType Name? VariablesDefinition? Directives? SelectionSet
            //   2. SelectionSet  (shorthand / anonymous query syntax, e.g. `{ foo bar }`)
            // In the shorthand form the only child is a SelectionSet, and the operation
            // is treated as an anonymous, untyped `query`.
            let is_shorthand = pair
                .clone()
                .into_inner()
                .next()
                .is_some_and(|first| first.is_rule(Rule::SelectionSet));
            if is_shorthand {
                let selection_set = pair.only_child();
                ExecutableDefinitionExt::OperationDefinition(OperationDefinition {
                    position,
                    description: None,
                    operation_type: OperationType::Query,
                    name: None,
                    variables_definition: None,
                    directives: vec![],
                    selection_set: build_selection_set(selection_set, file),
                })
            } else {
                let (
                    description,
                    operation_type,
                    name,
                    variables_definition,
                    directives,
                    selection_set,
                ) = parts!(
                    pair,
                    Description opt,
                    OperationType,
                    Name opt,
                    VariablesDefinition opt,
                    Directives opt,
                    SelectionSet
                );
                ExecutableDefinitionExt::OperationDefinition(OperationDefinition {
                    position,
                    description: description.map(|pair| build_description(pair, file)),
                    operation_type: str_to_operation_type(operation_type.as_str()),
                    name: name.map(|pair| pair.to_ident(file)),
                    variables_definition: variables_definition
                        .map(|pair| build_variables_definition(pair, file)),
                    directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
                    selection_set: build_selection_set(selection_set, file),
                })
            }
        }
        Rule::FragmentDefinition => {
            let (description, _, name, type_condition, directives, selection_set) = parts!(
                pair,
                Description opt,
                KEYWORD_fragment,
                FragmentName,
                TypeCondition,
                Directives opt,
                SelectionSet
            );
            ExecutableDefinitionExt::FragmentDefinition(FragmentDefinition {
                position,
                description: description.map(|pair| build_description(pair, file)),
                name: name.to_ident(file),
                type_condition: {
                    let (_, name) = parts!(type_condition, KEYWORD_on, NamedType);
                    name.to_ident(file)
                },
                directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
                selection_set: build_selection_set(selection_set, file),
            })
        }
        Rule::ext_ImportStatement => {
            // pair becomes ext_ImportStatementContent
            let pair = pair.only_child();
            let (_, targets, _, path) = parts!(
                pair,
                ext_KEYWORD_import,
                ext_ImportTargets,
                ext_KEYWORD_from,
                StringValue
            );
            ExecutableDefinitionExt::Import(ImportDefinition {
                position,
                targets: targets
                    .into_inner()
                    .map(|pair| {
                        if pair.is_rule(Rule::Name) {
                            ImportTarget::Name(pair.to_ident(file))
                        } else {
                            // "*"
                            ImportTarget::Wildcard
                        }
                    })
                    .collect(),
                path: build_string_value(path, file),
            })
        }
        rule => panic!("Unexpected {:?} as a child of ExecutableDefinition", rule),
    }
}

pub fn str_to_operation_type(o: &str) -> OperationType {
    match o {
        "query" => OperationType::Query,
        "mutation" => OperationType::Mutation,
        "subscription" => OperationType::Subscription,
        _ => panic!("Unknown operation type {}", o),
    }
}
