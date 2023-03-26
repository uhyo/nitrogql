//! Module for reading introspection json (result of the standard introspection query) into the schema object.

use std::borrow::Cow;

use error::IntrospectionError;
use graphql_type_system::Schema;

mod error;
mod introspection;
#[cfg(test)]
mod tests;

use introspection::IntrospectionResult;

pub fn schema_from_introspection_json<D: Default>(
    source: &str,
) -> Result<Schema<Cow<str>, D>, IntrospectionError> {
    let json: IntrospectionResult = serde_json::from_str(source)?;
    introspection::introspection(&json)
}
