/// Representation of a scalar type's TypeScript type
/// as defined in the config file.
#[derive(Debug)]
pub enum ScalarTypeConfig {
    /// Single specification for use in all situations.
    Single(String),
}

impl ScalarTypeConfig {
    /// Get the TypeScript type as a resolver output type.
    pub fn as_resolver_output_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as a resolver input type.
    pub fn as_resolver_input_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as an operation output type.
    pub fn as_operation_output_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as an operation input type.
    pub fn as_operation_input_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
}
