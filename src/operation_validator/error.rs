use std::fmt::Display;

use thiserror::Error;

use crate::{
    error::PositionedError,
    graphql_parser::ast::{base::Pos, operations::OperationType},
};

#[derive(Debug)]
pub struct CheckOperationError {
    position: Pos,
    message: CheckOperationErrorMessage,
    additional_info: Vec<(Pos, CheckOperationErrorMessage)>,
}

impl CheckOperationError {
    pub fn with_additional_info(
        mut self,
        infos: impl IntoIterator<Item = (Pos, CheckOperationErrorMessage)>,
    ) -> Self {
        self.additional_info.extend(infos);
        self
    }
}

impl From<CheckOperationError> for PositionedError {
    fn from(value: CheckOperationError) -> Self {
        PositionedError::new(
            value.message.into(),
            Some(value.position),
            value
                .additional_info
                .into_iter()
                .map(|(pos, err)| (pos, err.to_string()))
                .collect(),
        )
    }
}

#[derive(Error, Debug)]
pub enum CheckOperationErrorMessage {
    #[error("Unnamed operation must be the only operation in this document")]
    UnNamedOperationMustBeSingle,
    #[error("Duplicate {} name", operation_type.as_str())]
    DuplicateOperationName { operation_type: OperationType },
    #[error("Duplicate fragment name")]
    DuplicateFragmentName { other_position: Pos },
    #[error("Root type for {} operation is not defined", operation_type.as_str())]
    NoRootType { operation_type: OperationType },
    #[error("Type '{name}' not found")]
    TypeNotFound { name: String },
    #[error("Directive '{name}' not found")]
    DirectiveNotFound { name: String },
    #[error("Directive '{name}' is not allowed for this location")]
    DirectiveLocationNotAllowed { name: String },
    #[error("Repeated application of directive '{name}' is not allowed")]
    RepeatedDirective { name: String },
    #[error("Cannot select fields of {kind} '{name}'")]
    SelectionOnInvalidType { kind: TypeKind, name: String },
    #[error("Field '{field_name}' is not found on type '{type_name}'")]
    FieldNotFound {
        field_name: String,
        type_name: String,
    },
    // Error that should be checked in type system check phase
    #[error("Type system error. This is a bug of checker")]
    TypeSystemError,
    // For additional info
    #[error("Another definition of '{name}'")]
    AnotherDefinitionPos { name: String },
    #[error("Definition of '{name}'")]
    DefinitionPos { name: String },
    #[error("Root types are defined here")]
    RootTypesAreDefinedHere,
}

impl CheckOperationErrorMessage {
    pub fn with_pos(self, position: Pos) -> CheckOperationError {
        CheckOperationError {
            position,
            message: self,
            additional_info: vec![],
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
