use std::fmt::Display;

use thiserror::Error;

use crate::{
    error::PositionedError,
    graphql_parser::ast::{base::Pos, operations::OperationType},
};

#[derive(Debug)]
pub struct CheckError {
    pub position: Pos,
    pub message: CheckErrorMessage,
    pub additional_info: Vec<(Pos, CheckErrorMessage)>,
}

impl CheckError {
    pub fn with_additional_info(
        mut self,
        infos: impl IntoIterator<Item = (Pos, CheckErrorMessage)>,
    ) -> Self {
        self.additional_info.extend(infos);
        self
    }
}

#[derive(Error, Debug)]
pub enum CheckErrorMessage {
    // errors for both
    #[error("Directive '{name}' is not defined")]
    UnknownDirective { name: String },
    #[error("Directive '{name}' is not allowed for this location")]
    DirectiveLocationNotAllowed { name: String },
    #[error("Repeated application of directive '{name}' is not allowed")]
    RepeatedDirective { name: String },
    #[error("This {kind} does not accept arguments")]
    ArgumentsNotNeeded { kind: &'static str },
    #[error("Required argument '{name}' is not specified")]
    RequiredArgumentNotSpecified { name: String },
    #[error("This value does not match expected type '{r#type}' ")]
    TypeMismatch { r#type: String },
    #[error("Variable '${name}' is not defined")]
    UnknownVariable { name: String },
    #[error("'{member}' is a member of enum {r#enum}")]
    UnknownEnumMember { member: String, r#enum: String },
    #[error("Argument '{name}' is not defined")]
    UnknownArgument { name: String },
    #[error("Field '{name}' is required")]
    RequiredFieldNotSpecified { name: String },
    #[error("Field '{name}' is not defined")]
    UnknownField { name: String },
    // errors for type system
    #[error("Name that starts with '__' is reserved")]
    UnscoUnsco,
    #[error("Name '{name}' is duplicated")]
    DuplicatedName { name: String },
    #[error("Type '{name}' is not defined")]
    UnknownType { name: String },
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
    // errors for operation
    #[error("Unnamed operation must be the only operation in this document")]
    UnNamedOperationMustBeSingle,
    #[error("Duplicate {} name", operation_type.as_str())]
    DuplicateOperationName { operation_type: OperationType },
    #[error("Duplicate fragment name")]
    DuplicateFragmentName { other_position: Pos },
    #[error("Root type for {} operation is not defined", operation_type.as_str())]
    NoRootType { operation_type: OperationType },
    #[error("Cannot select fields of {kind} '{name}'")]
    SelectionOnInvalidType { kind: TypeKind, name: String },
    #[error("Field '{field_name}' is not found on type '{type_name}'")]
    FieldNotFound {
        field_name: String,
        type_name: String,
    },
    #[error("Selection '{name}' is duplicated")]
    DuplicateSelectionName { name: String },
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
    #[error("Duplicated variable name '{name}'")]
    DuplicatedVariableName { name: String },
}

impl CheckErrorMessage {
    pub fn with_pos(self, position: Pos) -> CheckError {
        CheckError {
            position,
            message: self,
            additional_info: vec![],
        }
    }
}

impl From<CheckError> for PositionedError {
    fn from(value: CheckError) -> Self {
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
