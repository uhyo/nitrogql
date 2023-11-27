use nitrogql_ast::{base::Ident, value::StringValue};

/// Resolved extension to operation document.
#[derive(Debug, Clone)]
pub struct OperationExtension<'src> {
    /// List of imports.
    pub imports: Vec<Import<'src>>,
}

#[derive(Debug, Clone)]
pub struct Import<'src> {
    /// Path to import from. (not resolved)
    pub path: StringValue,
    pub targets: ImportTargets<'src>,
}

#[derive(Debug, Clone)]
pub enum ImportTargets<'src> {
    Wildcard,
    Specific(Vec<Ident<'src>>),
}
