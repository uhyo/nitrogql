#[derive(Debug)]
pub struct ResolverTypePrinterOptions {
    /// Name of root resolver type.
    pub root_resolver_type: String,
}

impl Default for ResolverTypePrinterOptions {
    fn default() -> Self {
        ResolverTypePrinterOptions {
            root_resolver_type: "Resolvers".into(),
        }
    }
}
