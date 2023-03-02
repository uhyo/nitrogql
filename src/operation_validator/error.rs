use std::fmt::Display;

use thiserror::Error;

use crate::graphql_parser::ast::{base::Pos, operations::OperationType};

#[derive(Debug)]
pub struct CheckOperationError {
    position: Pos,
    message: CheckOperationErrorMessage,
}

#[derive(Error, Debug)]
pub enum CheckOperationErrorMessage {
    #[error("Unnamed operation must be the only operation in this document")]
    UnNamedOperationMustBeSingle,
    #[error("Duplicate {} name", operation_type.as_str())]
    DuplicateOperationName {
        operation_type: OperationType,
        other_position: Pos,
    },
    #[error("Duplicate fragment name")]
    DuplicateFragmentName { other_position: Pos },
    #[error("Root type for {} operation is not defined", operation_type.as_str())]
    NoRootType {
        operation_type: OperationType,
        schema_definition: Pos,
    },
    #[error("Type '{name}' not found")]
    TypeNotFound { name: String },
    #[error("Cannot select fields of {kind} '{name}'")]
    SelectionOnInvalidType {
        kind: TypeKind,
        name: String,
        type_def: Pos,
    },
    #[error("Field '{field_name}' is not found on type '{type_name}'")]
    FieldNotFound {
        field_name: String,
        type_name: String,
        type_def: Pos,
    },
}

impl CheckOperationErrorMessage {
    pub fn with_pos(self, position: Pos) -> CheckOperationError {
        CheckOperationError {
            position,
            message: self,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
}

impl Display for TypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeKind::Scalar => write!(f, "scalar"),
            TypeKind::Object => write!(f, "object"),
            TypeKind::Interface => write!(f, "interface"),
            TypeKind::Union => write!(f, "union"),
            TypeKind::Enum => write!(f, "enum"),
            TypeKind::InputObject => write!(f, "input object"),
        }
    }
}
