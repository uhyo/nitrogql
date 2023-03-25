//! Module for reading introspection json (result of the standard introspection query) into the schema object.

use error::IntrospectionError;
use graphql_type_system::Schema;
use json_to_value::GraphQLValue;

mod error;
mod introspection;
mod json_to_value;
#[cfg(test)]
mod tests;

pub use json_to_value::json_to_value;

/// TODO: make this receive &str
pub fn schema_from_introspection_json(
    source: &GraphQLValue<String>,
) -> Result<Schema<&str, ()>, IntrospectionError> {
    Ok(introspection::introspection(source))
}
