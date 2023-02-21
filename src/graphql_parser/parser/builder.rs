//! This module builds ast from raw parser result.
//!

use crate::{
    graphql_parser::ast::{operations::OperationType, OperationDocument},
    parts,
};

use self::{
    directives::build_directives,
    operation::{build_executable_definition, build_variables_definition},
    selection_set::build_selection_set,
    utils::PairExt,
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
