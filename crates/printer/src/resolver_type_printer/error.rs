use thiserror::Error;

use nitrogql_ast::base::Pos;

#[derive(Error, Debug)]
pub enum ResolverTypePrinterError {
    #[error("Type for scalar '{name}' is not provided")]
    ScalarTypeNotProvided { position: Pos, name: String },
}

pub type ResolverTypePrinterResult<T> = Result<T, ResolverTypePrinterError>;
