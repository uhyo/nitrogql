use thiserror::Error;

use crate::ast::base::Pos;

#[derive(Error, Debug)]
pub enum SchemaTypePrinterError {
    #[error("Type for scalar '{name}' is not provided")]
    ScalarTypeNotProvided { position: Pos, name: String },
}

pub type SchemaTypePrinterResult<T> = Result<T, SchemaTypePrinterError>;
