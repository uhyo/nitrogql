//! This module builds ast from raw parser result.
//!

use crate::{
    graphql_parser::ast::{operations::OperationType, OperationDefinition, OperationDocument},
    parts,
};

use self::{
    directives::build_directives, operation::build_variables_definition,
    selection_set::build_selection_set, utils::PairExt,
};

use super::Rule;
use pest::iterators::{Pair, Pairs};

mod base;
mod directives;
mod operation;
mod selection_set;
mod r#type;
mod utils;
mod value;

pub fn build_operation_document(pairs: Pairs<Rule>) -> OperationDocument {
    for pair in pairs {
        match pair.as_rule() {
            Rule::ExecutableDocument => {
                let definitions: Vec<_> = pair
                    .into_inner()
                    .filter(|pair| pair.is_rule(Rule::ExecutableDefinition))
                    .map(|pair| build_executable_definition(pair))
                    .collect();
                return OperationDocument { definitions };
            }
            rule => panic!("Unexpected Rule {:?}", rule),
        }
    }
    panic!("Empty document")
}

fn build_executable_definition(pair: Pair<Rule>) -> OperationDefinition {
    let pair = pair.only_child();
    // TODO: handling of OperationSet (abbreviated syntax)
    let (operation_type, name, variables_definition, directives, selection_set) = parts!(
        pair.into_inner(),
        OperationType,
        Name opt,
        VariablesDefinition opt,
        Directives opt,
        SelectionSet
    );
    OperationDefinition {
        operation_type: str_to_operation_type(operation_type.as_str()),
        name: name.map(|pair| pair.into()),
        variables_definition: variables_definition.map(build_variables_definition),
        directives: directives.map_or(vec![], build_directives),
        selection_set: build_selection_set(selection_set),
    }
}

fn str_to_operation_type(o: &str) -> OperationType {
    match o {
        "query" => OperationType::Query,
        "mutation" => OperationType::Mutation,
        "subscription" => OperationType::Subscription,
        _ => panic!("Unknown operation type {}", o),
    }
}
