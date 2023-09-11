use std::collections::HashMap;

use nitrogql_ast::TypeSystemDocument;

use crate::ts_types::TSType;

pub trait ResolverTypePrinterPlugin {
    /// Transform resolver output types.
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        base: HashMap<&'src str, TSType>,
    ) -> HashMap<&'src str, TSType>;
}
