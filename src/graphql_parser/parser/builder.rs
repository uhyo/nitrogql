//! This module builds ast from raw parser result.
//!

use crate::graphql_parser::ast::{OperationDefinition, OperationDocument};

use super::Rule;
use pest::iterators::{Pair, Pairs};

pub fn build_operation_document(pairs: Pairs<Rule>) -> OperationDocument {
    for pair in pairs {
        match pair.as_rule() {
            Rule::ExecutableDocument => {
                let definitions: Vec<_> = pair
                    .into_inner()
                    .map(|pair| build_operation_definition(pair))
                    .collect();
                return OperationDocument { definitions };
            }
            rule => panic!("Unexpected Rule {:?}", rule),
        }
    }
    panic!("Empty document")
}

fn build_operation_definition(pair: Pair<Rule>) -> OperationDefinition {
    OperationDefinition {
        source: pair.as_str(),
    }
}
