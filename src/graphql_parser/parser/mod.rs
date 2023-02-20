use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

use self::builder::build_operation_document;

use super::ast::OperationDocument;

mod builder;

#[derive(Parser)]
#[grammar = "graphql_parser/parser/grammar.pest"]
pub struct RawParser;

pub fn parse_operation_document(document: &str) -> Result<OperationDocument> {
    let res = RawParser::parse(Rule::ExecutableDocument, document)?;

    Ok(build_operation_document(res))
}
