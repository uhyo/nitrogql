use std::collections::HashMap;

use nitrogql_ast::TypeSystemDocument;

use crate::{ts_types::TSType, ResolverTypePrinterOptions};

/// A plugin that can transform resolver output types.
pub trait ResolverTypePrinterPlugin {
    /// Transform resolver output types.
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        options: &ResolverTypePrinterOptions,
        base: HashMap<&'src str, TSType>,
    ) -> HashMap<&'src str, TSType>;
    /// Transform document so that it represents which fields
    /// have resolvers.
    fn transform_document_for_resolvers<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> TypeSystemDocument<'src>;
}
