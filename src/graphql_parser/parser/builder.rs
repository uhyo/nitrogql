//! This module builds ast from raw parser result.
//!

use crate::graphql_parser::ast::{OperationDocument, TypeSystemOrExtensionDocument};

use self::{
    operation::build_executable_definition, type_system::build_type_system_definition_or_extension,
    utils::PairExt,
};

use super::Rule;
use pest::iterators::Pairs;

mod base;
mod directives;
mod operation;
mod selection_set;
mod r#type;
mod type_system;
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

pub fn build_type_system_or_extension_document(
    pairs: Pairs<Rule>,
) -> TypeSystemOrExtensionDocument {
    for pair in pairs {
        match pair.as_rule() {
            Rule::TypeSystemExtensionDocument => {
                let definitions: Vec<_> = pair
                    .into_inner()
                    .filter(|pair| pair.is_rule(Rule::TypeSystemDefinitionOrExtension))
                    .map(|pair| build_type_system_definition_or_extension(pair))
                    .collect();
                return TypeSystemOrExtensionDocument { definitions };
            }
            rule => panic!("Unexpected Rule {:?}", rule),
        }
    }
    panic!("Empty document")
}
