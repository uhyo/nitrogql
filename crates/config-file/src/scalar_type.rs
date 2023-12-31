use serde::Deserialize;

use crate::TypeTarget;

/// Representation of a scalar type's TypeScript type
/// as defined in the config file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarTypeConfig {
    /// Single specification for use in all situations.
    Single(String),
}

impl ScalarTypeConfig {
    /// Get the TypeScript type for given target.
    pub fn get_type(&self, _target: TypeTarget) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as a resolver output type.
    pub fn as_resolver_output_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as a resolver input type.
    pub fn as_resolver_input_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as an operation output type.
    pub fn as_operation_output_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Get the TypeScript type as an operation input type.
    pub fn as_operation_input_type(&self) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
        }
    }
    /// Returns an Iterator over all type names used in this config.
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        match self {
            ScalarTypeConfig::Single(type_name) => std::iter::once(type_name.as_str()),
        }
    }
}

impl<'de> Deserialize<'de> for ScalarTypeConfig {
    fn deserialize<D>(deserializer: D) -> Result<ScalarTypeConfig, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ScalarTypeConfig::Single(s))
    }
}
