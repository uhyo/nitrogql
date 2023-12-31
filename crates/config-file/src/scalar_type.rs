use serde::Deserialize;

use crate::TypeTarget;

/// Representation of a scalar type's TypeScript type
/// as defined in the config file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum ScalarTypeConfig {
    /// Single specification for use in all situations.
    Single(String),
    /// Specification as a pair of send type and receive type.
    SendReceive(SendReceiveScalarTypeConfig),
    /// Specification as four diffrent types.
    Separate(SeparateScalarTypeConfig),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendReceiveScalarTypeConfig {
    pub send: String,
    pub receive: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeparateScalarTypeConfig {
    pub resolver_output: String,
    pub resolver_input: String,
    pub operation_output: String,
    pub operation_input: String,
}

impl ScalarTypeConfig {
    /// Get the TypeScript type for given target.
    pub fn get_type(&self, target: TypeTarget) -> &str {
        match self {
            ScalarTypeConfig::Single(type_name) => type_name,
            ScalarTypeConfig::SendReceive(config) => match target {
                TypeTarget::ResolverOutput => &config.send,
                TypeTarget::ResolverInput => &config.receive,
                TypeTarget::OperationOutput => &config.receive,
                TypeTarget::OperationInput => &config.send,
            },
            ScalarTypeConfig::Separate(config) => match target {
                TypeTarget::ResolverOutput => &config.resolver_output,
                TypeTarget::ResolverInput => &config.resolver_input,
                TypeTarget::OperationOutput => &config.operation_output,
                TypeTarget::OperationInput => &config.operation_input,
            },
        }
    }
    /// Returns an Iterator over all type names used in this config.
    pub fn type_names(&self) -> impl Iterator<Item = &str> {
        match self {
            ScalarTypeConfig::Single(type_name) => vec![(type_name.as_str())].into_iter(),
            ScalarTypeConfig::SendReceive(config) => {
                vec![config.send.as_str(), config.receive.as_str()].into_iter()
            }
            ScalarTypeConfig::Separate(config) => vec![
                config.resolver_output.as_str(),
                config.resolver_input.as_str(),
                config.operation_output.as_str(),
                config.operation_input.as_str(),
            ]
            .into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_type_config() {
        let config = ScalarTypeConfig::Single("string".to_string());
        assert_eq!(config.get_type(TypeTarget::ResolverOutput), "string");
        assert_eq!(config.get_type(TypeTarget::ResolverInput), "string");
        assert_eq!(config.get_type(TypeTarget::OperationOutput), "string");
        assert_eq!(config.get_type(TypeTarget::OperationInput), "string");

        let config = ScalarTypeConfig::SendReceive(SendReceiveScalarTypeConfig {
            send: "string".to_string(),
            receive: "number".to_string(),
        });
        assert_eq!(config.get_type(TypeTarget::ResolverOutput), "string");
        assert_eq!(config.get_type(TypeTarget::ResolverInput), "number");
        assert_eq!(config.get_type(TypeTarget::OperationOutput), "number");
        assert_eq!(config.get_type(TypeTarget::OperationInput), "string");

        let config = ScalarTypeConfig::Separate(SeparateScalarTypeConfig {
            resolver_output: "string".to_string(),
            resolver_input: "number".to_string(),
            operation_output: "boolean".to_string(),
            operation_input: "bigint".to_string(),
        });
        assert_eq!(config.get_type(TypeTarget::ResolverOutput), "string");
        assert_eq!(config.get_type(TypeTarget::ResolverInput), "number");
        assert_eq!(config.get_type(TypeTarget::OperationOutput), "boolean");
        assert_eq!(config.get_type(TypeTarget::OperationInput), "bigint");
    }

    #[test]
    fn test_scalar_type_config_type_names() {
        let config = ScalarTypeConfig::Single("string".to_string());
        assert_eq!(
            config.type_names().collect::<Vec<_>>(),
            vec!["string"].into_iter().collect::<Vec<_>>()
        );

        let config = ScalarTypeConfig::SendReceive(SendReceiveScalarTypeConfig {
            send: "string".to_string(),
            receive: "number".to_string(),
        });
        assert_eq!(
            config.type_names().collect::<Vec<_>>(),
            vec!["string", "number"].into_iter().collect::<Vec<_>>()
        );

        let config = ScalarTypeConfig::Separate(SeparateScalarTypeConfig {
            resolver_output: "string".to_string(),
            resolver_input: "number".to_string(),
            operation_output: "boolean".to_string(),
            operation_input: "bigint".to_string(),
        });
        assert_eq!(
            config.type_names().collect::<Vec<_>>(),
            vec!["string", "number", "boolean", "bigint"]
                .into_iter()
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn parsing() {
        let config: ScalarTypeConfig = serde_json::from_str(r#""string""#).unwrap();
        assert_eq!(config, ScalarTypeConfig::Single("string".to_string()));

        let config: ScalarTypeConfig = serde_json::from_str(
            r#"{
                "send": "string",
                "receive": "number"
            }"#,
        )
        .unwrap();
        assert_eq!(
            config,
            ScalarTypeConfig::SendReceive(SendReceiveScalarTypeConfig {
                send: "string".to_string(),
                receive: "number".to_string(),
            })
        );

        let config: ScalarTypeConfig = serde_json::from_str(
            r#"{
                "resolverOutput": "string",
                "resolverInput": "number",
                "operationOutput": "boolean",
                "operationInput": "bigint"
            }"#,
        )
        .unwrap();
        assert_eq!(
            config,
            ScalarTypeConfig::Separate(SeparateScalarTypeConfig {
                resolver_output: "string".to_string(),
                resolver_input: "number".to_string(),
                operation_output: "boolean".to_string(),
                operation_input: "bigint".to_string(),
            })
        );
    }
}
