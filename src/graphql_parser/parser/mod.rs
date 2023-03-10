use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use self::builder::{build_operation_document, build_type_system_or_extension_document};

use crate::ast::{OperationDocument, TypeSystemOrExtensionDocument};

mod builder;

#[derive(Parser)]
#[grammar = "graphql_parser/parser/grammar.pest"]
pub struct RawParser;

pub fn parse_operation_document(document: &str) -> Result<OperationDocument> {
    let res = RawParser::parse(Rule::ExecutableDocument, document)?;

    Ok(build_operation_document(res))
}

pub fn parse_type_system_document(document: &str) -> Result<TypeSystemOrExtensionDocument> {
    let res = RawParser::parse(Rule::TypeSystemExtensionDocument, document)?;

    Ok(build_type_system_or_extension_document(res))
}
