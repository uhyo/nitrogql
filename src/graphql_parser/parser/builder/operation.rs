use crate::{
    ast::{
        base::Pos,
        operations::{
            ExecutableDefinition, FragmentDefinition, OperationDefinition, OperationType,
            VariableDefinition, VariablesDefinition,
        },
    },
    parts,
};

use super::{
    base::build_variable, directives::build_directives, r#type::build_type,
    selection_set::build_selection_set, utils::PairExt, value::build_value, Rule,
};
use pest::iterators::Pair;

/// Parses a VariablesDefinition Pair.
pub fn build_variables_definition(pair: Pair<Rule>) -> VariablesDefinition {
    let position: Pos = (&pair).into();
    let defs = pair.all_children(Rule::VariableDefinition);
    VariablesDefinition {
        position,
        definitions: defs.into_iter().map(build_variable_definition).collect(),
    }
}

/// Parses one VariableDefinition Pair.
pub fn build_variable_definition(pair: Pair<Rule>) -> VariableDefinition {
    let pos: Pos = (&pair).into();
    let (variable, ty, default_value, directives) = parts!(
        pair,
        Variable,
        Type,
        DefaultValue opt,
        Directives opt
    );

    VariableDefinition {
        pos,
        name: build_variable(variable),
        r#type: build_type(ty),
        default_value: default_value.map(|pair| {
            let child = pair.only_child();
            build_value(child)
        }),
        directives: directives.map_or(vec![], build_directives),
    }
}

pub fn build_executable_definition(pair: Pair<Rule>) -> ExecutableDefinition {
    let pair = pair.only_child();
    let position = (&pair).into();
    match pair.as_rule() {
        Rule::OperationDefinition => {
            // TODO: handling of OperationSet (abbreviated syntax)
            let (operation_type, name, variables_definition, directives, selection_set) = parts!(
                pair,
                OperationType,
                Name opt,
                VariablesDefinition opt,
                Directives opt,
                SelectionSet
            );
            ExecutableDefinition::OperationDefinition(OperationDefinition {
                position,
                operation_type: str_to_operation_type(operation_type.as_str()),
                name: name.map(|pair| pair.into()),
                variables_definition: variables_definition.map(build_variables_definition),
                directives: directives.map_or(vec![], build_directives),
                selection_set: build_selection_set(selection_set),
            })
        }
        Rule::FragmentDefinition => {
            let (_, name, type_condition, directives, selection_set) = parts!(
                pair,
                KEYWORD_fragment,
                FragmentName,
                TypeCondition,
                Directives opt,
                SelectionSet
            );
            ExecutableDefinition::FragmentDefinition(FragmentDefinition {
                position,
                name: name.into(),
                type_condition: {
                    let (_, name) = parts!(type_condition, KEYWORD_on, NamedType);
                    name.into()
                },
                directives: directives.map_or(vec![], build_directives),
                selection_set: build_selection_set(selection_set),
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
