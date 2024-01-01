use std::collections::HashMap;

use nitrogql_config_file::{
    ScalarTypeConfig, SendReceiveScalarTypeConfig, SeparateScalarTypeConfig,
};

use crate::{PluginSchemaExtensions, PluginV1Beta};

mod tests;

#[derive(Debug, Default)]
pub struct GraphQLScalarsPlugin {
    scalar_extensions: HashMap<String, ScalarTypeConfig>,
}

impl PluginV1Beta for GraphQLScalarsPlugin {
    fn name(&self) -> &str {
        "nitrogql:graphql-scalars-plugin"
    }
    fn load_schema_extensions(&mut self, extensions: PluginSchemaExtensions) {
        for (type_name, extensions) in extensions.type_extensions {
            // @nitrogql/core adds a `nitrogql:kind` extension field.
            let kind = extensions.get("nitrogql:kind").and_then(|v| v.as_str());
            if kind != Some("scalar") {
                continue;
            }
            // GraphQL Scalars' scalar types has a `codegenScalarType` extension field.
            // nitrogql supports following forms:
            // - `codegenScalarType: "string"`
            // - `codegenScalarType: { send: "string", receive: "string" }`
            // - `codegenScalarType: { resolverInput: "string", resolverOutput: "string", operationInput: "string", operationOutput: "string" }`
            // - `codegenScalarType: { input: "string", output: "string" }`
            // The last one is for compatibility with graphql-codegen.
            // It is interpreted same as send/receive although the original meaning is different.
            let Some(codegen_scalar_type) = extensions.get("codegenScalarType") else {
                continue;
            };
            if let Some(codegen_scalar_type) = codegen_scalar_type.as_str() {
                self.scalar_extensions.insert(
                    type_name.clone(),
                    ScalarTypeConfig::Single(codegen_scalar_type.to_string()),
                );
                continue;
            }
            let Some(codegen_scalar_type) = codegen_scalar_type.as_mapping() else {
                continue;
            };
            if let (Some(send), Some(receive)) = (
                codegen_scalar_type
                    .get("send")
                    .or(codegen_scalar_type.get("input"))
                    .and_then(|v| v.as_str()),
                codegen_scalar_type
                    .get("receive")
                    .or(codegen_scalar_type.get("output"))
                    .and_then(|v| v.as_str()),
            ) {
                self.scalar_extensions.insert(
                    type_name.clone(),
                    ScalarTypeConfig::SendReceive(SendReceiveScalarTypeConfig {
                        send: send.to_string(),
                        receive: receive.to_string(),
                    }),
                );
                continue;
            }
            if let (
                Some(resolver_input),
                Some(resolver_output),
                Some(operation_input),
                Some(operation_output),
            ) = (
                codegen_scalar_type
                    .get("resolverInput")
                    .and_then(|v| v.as_str()),
                codegen_scalar_type
                    .get("resolverOutput")
                    .and_then(|v| v.as_str()),
                codegen_scalar_type
                    .get("operationInput")
                    .and_then(|v| v.as_str()),
                codegen_scalar_type
                    .get("operationOutput")
                    .and_then(|v| v.as_str()),
            ) {
                self.scalar_extensions.insert(
                    type_name.clone(),
                    ScalarTypeConfig::Separate(SeparateScalarTypeConfig {
                        resolver_input: resolver_input.to_string(),
                        resolver_output: resolver_output.to_string(),
                        operation_input: operation_input.to_string(),
                        operation_output: operation_output.to_string(),
                    }),
                );
                continue;
            }
        }
    }
    fn schema_addition(&self) -> Option<String> {
        let mut schema_addition = String::new();
        // Sort by type name to make the output deterministic.
        let mut scalar_extensions: Vec<_> = self.scalar_extensions.iter().collect();
        scalar_extensions.sort_by_key(|(type_name, _)| *type_name);
        for (type_name, codegen_scalar_type) in scalar_extensions {
            let types = codegen_scalar_type.separate_ref();
            schema_addition.push_str(&format!(
                "extend scalar {type_name} @nitrogql_ts_type(
        resolverInput: \"{}\"
        resolverOutput: \"{}\"
        operationInput: \"{}\"
        operationOutput: \"{}\"
    )\n",
                types.resolver_input,
                types.resolver_output,
                types.operation_input,
                types.operation_output,
            ));
        }
        if schema_addition.is_empty() {
            None
        } else {
            Some(schema_addition)
        }
    }
}
