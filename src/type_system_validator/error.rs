use thiserror::Error;

use crate::{error::PositionedError, graphql_parser::ast::base::Pos};

#[derive(Debug)]
pub struct CheckTypeSystemError {
    pub position: Pos,
    pub message: CheckTypeSystemErrorMessage,
    pub additional_info: Vec<(Pos, CheckTypeSystemErrorMessage)>,
}

#[derive(Error, Debug)]
pub enum CheckTypeSystemErrorMessage {
    #[error("Name that starts with '__' is reserved")]
    UnscoUnsco,
    #[error("Name '{name}' is duplicated")]
    DuplicatedName { name: String },
    #[error("Directive '{name}' is not defined")]
    UnknownDirective { name: String },
    #[error("Type '{name}' is not defined")]
    UnknownType { name: String },
    #[error("Directive '{name}' is not allowed for this location")]
    DirectiveLocationNotAllowed { name: String },
    #[error("Repeated application of directive '{name}' is not allowed")]
    RepeatedDirective { name: String },
    #[error("Directive '{name}' is recursing")]
    RecursingDirective { name: String },
    #[error("Output type '{name}' is not allowed here")]
    NoOutputType { name: String },
    #[error("Input type '{name}' is not allowed here")]
    NoInputType { name: String },
    #[error("'{name}' is not an interface")]
    NotInterface { name: String },
    #[error("This type must implement interface '{name}'")]
    InterfaceNotImplemented { name: String },
    #[error("Interface must not implement itself")]
    NoImplementSelf,
    #[error("This type must have a field '{field_name}' from interface '{interface_name}'")]
    InterfaceFieldNotImplemented {
        field_name: String,
        interface_name: String,
    },
    #[error(
        "Type of this argument does not match the same argument from interface '{interface_name}'"
    )]
    FieldTypeMisMatchWithInterface { interface_name: String },
    #[error("Type of this filed does not match the same field from interface '{interface_name}'")]
    InterfaceArgumentNotImplemented {
        argument_name: String,
        interface_name: String,
    },
    #[error(
        "Type of this argument does not match the same argument from interface '{interface_name}'"
    )]
    ArgumentTypeMisMatchWithInterface { interface_name: String },
    #[error(
        "Type of this argument must be nullable because it is not in the same field from interface '{interface_name}'"
    )]
    ArgumentTypeNonNullAgainstInterface { interface_name: String },
    #[error("'{member_name}' is not an object type")]
    NonObjectTypeUnionMember { member_name: String },
}

impl CheckTypeSystemErrorMessage {
    pub fn with_pos(self, position: Pos) -> CheckTypeSystemError {
        CheckTypeSystemError {
            position,
            message: self,
            additional_info: vec![],
        }
    }
}

impl From<CheckTypeSystemError> for PositionedError {
    fn from(value: CheckTypeSystemError) -> Self {
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
