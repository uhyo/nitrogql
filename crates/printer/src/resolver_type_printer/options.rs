use nitrogql_config_file::Config;

#[derive(Debug)]
pub struct ResolverTypePrinterOptions {
    /// Name of root resolver type.
    pub root_resolver_type: String,
    /// Source of schema type to import from.
    pub schema_source: String,
    /// Name of the root TypeScript namespace that contains schema types.
    pub schema_root_namespace: String,
}

impl Default for ResolverTypePrinterOptions {
    fn default() -> Self {
        ResolverTypePrinterOptions {
            root_resolver_type: "Resolvers".into(),
            schema_source: "".into(),
            schema_root_namespace: "Schema".into(),
        }
    }
}

impl ResolverTypePrinterOptions {
    pub fn from_config(_config: &Config) -> Self {
        ResolverTypePrinterOptions::default()
    }
}
