use nitrogql_ast::{TypeSystemOrExtensionDocument, base::Pos, operation_ext::OperationDocumentExt};
use nitrogql_error::{ErrorCode, PositionedError};
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

    fn from_pest(error: pest::error::Error<Rule>, document: &str, file: usize) -> Self {
        // convert 1-based line and column to 0-based
        let (line, col) = match error.line_col {
            pest::error::LineColLocation::Pos((line, column)) => (line - 1, column - 1),
            pest::error::LineColLocation::Span((line, column), _) => (line - 1, column - 1),
        };
        let span = match error.location {
            pest::error::InputLocation::Pos(start) => start..token_end(document, start),
            pest::error::InputLocation::Span((start, end)) => start..end,
        };
        let position = Pos::new(line, col, span, file);
        let message = error.variant.message().into_owned();

        ParseError { position, message }
    }
}

/// Returns the end of the token starting at given byte offset.
/// A token is either a name or a single character other than whitespace.
fn token_end(document: &str, start: usize) -> usize {
    let rest = &document[start..];
    let is_name_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
    match rest.chars().next() {
        Some(c) if is_name_char(c) => {
            start + rest.find(|c: char| !is_name_char(c)).unwrap_or(rest.len())
        }
        Some(c) if !c.is_whitespace() => start + c.len_utf8(),
        _ => start,
    }
}

impl From<ParseError> for PositionedError {
    fn from(value: ParseError) -> Self {
        let position = value.position;
        let additional_info = vec![];
        let inner = ParseErrorMessage(value.message).into();

        PositionedError::new(inner, Some(position), additional_info)
            .with_code(ErrorCode::SyntaxError)
    }
}

/// Parses an operation document.
/// `file` is the index of the file, recorded in every `Pos` of the result.
pub fn parse_operation_document(
    document: &str,
    file: usize,
) -> Result<OperationDocumentExt<'_>, ParseError> {
    let res = RawParser::parse(Rule::ExecutableDocument, document)
        .map_err(|error| ParseError::from_pest(error, document, file))?;

    Ok(build_operation_document(res, file))
}

/// Parses a type system document.
/// `file` is the index of the file, recorded in every `Pos` of the result.
pub fn parse_type_system_document(
    document: &str,
    file: usize,
) -> Result<TypeSystemOrExtensionDocument<'_>, ParseError> {
    let res = RawParser::parse(Rule::TypeSystemExtensionDocument, document)
        .map_err(|error| ParseError::from_pest(error, document, file))?;

    Ok(build_type_system_or_extension_document(res, file))
}
