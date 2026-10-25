use std::collections::HashMap;

use nitrogql_ast::{TypeSystemDocument, TypeSystemOrExtensionDocument};
use nitrogql_parser::{ParseError, parse_type_system_document};
use nitrogql_printer::{ResolverTypePrinterOptions, ResolverTypePrinterPlugin, ts_types::TSType};

use crate::{PluginV1Beta, plugin_v1::PluginSchemaExtensions};

pub use crate::PluginCheckResult;

pub use self::host::PluginHost;

mod host;

/// Wrapper of naked plugin.
#[derive(Debug)]
pub struct Plugin {
    /// The naked plugin.
    raw: Box<dyn PluginV1Beta>,
}

impl Plugin {
    /// Creates a new plugin.
    pub fn new(raw: Box<dyn PluginV1Beta>) -> Self {
        Self { raw }
    }

    /// Returns the name of the plugin.
    pub fn name(&self) -> &str {
        self.raw.name()
    }

    /// Load schema extensions.
    pub fn load_schema_extensions(&mut self, extensions: PluginSchemaExtensions) {
        self.raw.load_schema_extensions(extensions)
    }

    /// Returns additional schema definition provided by the plugin.
    /// The definition is loaded as a virtual file of the host.
    pub fn schema_addition<'src>(
        &self,
        host: &mut impl PluginHost<'src>,
    ) -> Result<Option<TypeSystemOrExtensionDocument<'src>>, ParseError> {
        let addition = self.raw.schema_addition();
        if let Some(addition) = addition {
            let (host_buf, file) = host.load_virtual_file(addition);
            let parsed = parse_type_system_document(host_buf, file)?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Checks schema.
    pub fn check_schema(&self, schema: &TypeSystemDocument) -> PluginCheckResult {
        self.raw.check_schema(schema)
    }

    pub fn transform_document_for_runtime_server<'s>(
        &self,
        document: &TypeSystemDocument<'s>,
    ) -> Option<TypeSystemDocument<'s>> {
        self.raw.transform_document_for_runtime_server(document)
    }
}

impl ResolverTypePrinterPlugin for Plugin {
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        options: &ResolverTypePrinterOptions,
        base: HashMap<&'src str, TSType>,
    ) -> HashMap<&'src str, TSType> {
        self.raw
            .transform_resolver_output_types(document, options, base)
    }
    fn transform_document_for_resolvers<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
    ) -> Option<TypeSystemDocument<'src>> {
        self.raw.transform_document_for_resolvers(document)
    }
}
