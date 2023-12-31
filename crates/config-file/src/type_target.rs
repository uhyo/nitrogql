use std::fmt::Display;

#[derive(Copy, Clone)]
pub enum TypeTarget {
    OperationInput,
    OperationOutput,
    ResolverInput,
    ResolverOutput,
}

impl TypeTarget {
    /// Returns the string representation of self.
    /// String representation is prefixed with `__` for use in type definitions.
    pub fn as_str(&self) -> &'static str {
        match self {
            TypeTarget::OperationInput => "__OperationInput",
            TypeTarget::OperationOutput => "__OperationOutput",
            TypeTarget::ResolverInput => "__ResolverInput",
            TypeTarget::ResolverOutput => "__ResolverOutput",
        }
    }
}

impl Display for TypeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
