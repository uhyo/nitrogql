use nitrogql_ast::{TypeSystemOrExtensionDocument, base::Pos, operation_ext::OperationDocumentExt};
use nitrogql_error::PositionedError;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use self::builder::{build_operation_document, build_type_system_or_extension_document};

mod builder;

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct RawParser;

#[derive(Error, Debug)]
#[error("Parse error: {0}")]
pub struct ParseErrorMessage(String);

/// Struct that expresses parse error.
#[derive(Debug)]
pub struct ParseError {
    position: Pos,
    message: String,
}

impl ParseError {
    /// Extracts error message.
    pub fn into_message(self) -> String {
        self.message
    }
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(error: pest::error::Error<Rule>) -> Self {
        // convert 1-based line and column to 0-based
        let (line, col) = match error.line_col {
            pest::error::LineColLocation::Pos((line, column)) => (line - 1, column - 1),
            pest::error::LineColLocation::Span((line, column), _) => (line - 1, column - 1),
        };
        let position = Pos::new(line, col);
        let message = error.variant.message().into_owned();

        ParseError { position, message }
    }
}

impl From<ParseError> for PositionedError {
    fn from(value: ParseError) -> Self {
        let position = value.position;
        let additional_info = vec![];
        let inner = ParseErrorMessage(value.message).into();

        PositionedError::new(inner, Some(position), additional_info)
    }
}

pub fn parse_operation_document(document: &str) -> Result<OperationDocumentExt, ParseError> {
    let res = RawParser::parse(Rule::ExecutableDocument, document)?;

    Ok(build_operation_document(res))
}

pub fn parse_type_system_document(
    document: &str,
) -> Result<TypeSystemOrExtensionDocument, ParseError> {
    let res = RawParser::parse(Rule::TypeSystemExtensionDocument, document)?;

    Ok(build_type_system_or_extension_document(res))
}
