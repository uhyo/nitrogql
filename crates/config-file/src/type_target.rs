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
    pub fn as_str(&self) -> &'static str {
        match self {
            TypeTarget::OperationInput => "OperationInput",
            TypeTarget::OperationOutput => "OperationOutput",
            TypeTarget::ResolverInput => "ResolverInput",
            TypeTarget::ResolverOutput => "ResolverOutput",
        }
    }
}

impl Display for TypeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
