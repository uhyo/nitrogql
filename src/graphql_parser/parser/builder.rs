//! This module builds ast from raw parser result.
//!

use crate::{
    graphql_parser::ast::{OperationDefinition, OperationDocument},
    parts,
};

use self::utils::PairExt;

use super::Rule;
use pest::iterators::{Pair, Pairs};

mod utils;

pub fn build_operation_document(pairs: Pairs<Rule>) -> OperationDocument {
    for pair in pairs {
        match pair.as_rule() {
            Rule::ExecutableDocument => {
                let definitions: Vec<_> = pair
                    .into_inner()
                    .filter(|pair| pair.as_rule() == Rule::ExecutableDefinition)
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
        source: operation_type.as_str(),
    }
}
