use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntrospectionError {
    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
    #[error("Invalid JSON value: {0}")]
    GraphQLError(String),
    #[error("Introspection type system error: {0}")]
    Introspection(String),
}
