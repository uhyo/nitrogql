use nitrogql_ast::base::Pos;
use nitrogql_error::{ErrorCode, PositionedError};
use thiserror::Error;

/// Error from the operation type printer.
///
/// The printer assumes that the document is valid against the schema,
/// so an error usually means that the document has not been checked or that the checker has missed something.
#[derive(Debug)]
pub struct OperationTypePrinterError {
    pub position: Pos,
    pub message: OperationTypePrinterErrorMessage,
}

#[derive(Error, Debug)]
pub enum OperationTypePrinterErrorMessage {
    #[error("Type '{name}' is not defined")]
    UnknownType { name: String },
    #[error("Type '{name}' does not have fields")]
    NotSelectable { name: String },
    #[error("Member '{name}' of a union is not an object type")]
    UnionMemberNotObject { name: String },
    #[error("Field '{field}' is not defined on type '{type_name}'")]
    UnknownField { field: String, type_name: String },
    #[error("Fragment '{name}' is not defined")]
    UnknownFragment { name: String },
    #[error("Directive '@{name}' needs an 'if' argument")]
    MissingIfArgument { name: String },
    #[error("Value of variable '${name}' is unknown")]
    UnknownVariable { name: String },
    #[error("Cannot merge selections of field '{name}' that have different shapes")]
    FieldShapeMismatch { name: String },
}

impl OperationTypePrinterErrorMessage {
    pub fn with_pos(self, position: Pos) -> OperationTypePrinterError {
        OperationTypePrinterError {
            position,
            message: self,
        }
    }
}

pub type OperationTypePrinterResult<T> = Result<T, OperationTypePrinterError>;

impl From<OperationTypePrinterError> for PositionedError {
    fn from(value: OperationTypePrinterError) -> Self {
        // The checker should have reported the error, so this is a bug of nitrogql.
        PositionedError::new(value.message.into(), Some(value.position), vec![])
            .with_code(ErrorCode::InternalError)
    }
}
