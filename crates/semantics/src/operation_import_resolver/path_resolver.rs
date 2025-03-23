use std::path::Path;

use nitrogql_ast::OperationDocument;

use crate::OperationExtension;

/// Trait for resolving paths to operations.
pub trait OperationResolver<'src> {
    /// Resolve a full path to an operation.
    fn resolve(&self, path: &Path)
    -> Option<(&OperationDocument<'src>, &OperationExtension<'src>)>;
}
