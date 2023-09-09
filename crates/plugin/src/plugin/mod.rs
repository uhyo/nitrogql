use std::cell::RefCell;

use nitrogql_ast::{TypeSystemDocument, TypeSystemOrExtensionDocument};
use nitrogql_parser::{parse_type_system_document, ParseError};

use crate::PluginV1Beta;

pub use crate::PluginCheckResult;

pub use self::host::PluginHost;

mod host;

/// Wrapper of naked plugin.
#[derive(Debug)]
pub struct Plugin<'host> {
    /// The naked plugin.
    raw: Box<dyn PluginV1Beta>,
    /// Parsed schema addition.
    parsed_schema_addition: RefCell<Option<TypeSystemOrExtensionDocument<'host>>>,
}

impl<'host> Plugin<'host> {
    /// Creates a new plugin.
    pub fn new(raw: Box<dyn PluginV1Beta>) -> Self {
        Self {
            raw,
            parsed_schema_addition: RefCell::new(None),
        }
    }

    /// Returns the name of the plugin.
    pub fn name(&self) -> &str {
        self.raw.name()
    }

    /// Returns additional schema definition provided by the plugin.
    pub fn schema_addition(
        &self,
        host: &mut impl PluginHost<'host>,
    ) -> Result<Option<TypeSystemOrExtensionDocument<'host>>, ParseError> {
        let mut cached = self.parsed_schema_addition.borrow_mut();
        if let Some(cached) = &*cached {
            return Ok(Some(cached.clone()));
        }
        let addition = self.raw.schema_addition();
        if let Some(addition) = addition {
            let host_buf = host.load_virtual_file(addition);
            let parsed = parse_type_system_document(host_buf)?;
            *cached = Some(parsed.clone());
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Checks schema.
    pub fn check_schema(&self, schema: &TypeSystemDocument) -> PluginCheckResult {
        self.raw.check_schema(schema)
    }
}
