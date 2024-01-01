use std::collections::HashMap;

use crate::{PluginSchemaExtensions, PluginV1Beta};

mod tests;

#[derive(Debug, Default)]
pub struct GraphQLScalarsPlugin {
    scalar_extensions: HashMap<String, String>,
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
            let codegen_scalar_type = extensions.get("codegenScalarType").and_then(|v| v.as_str());
            if let Some(codegen_scalar_type) = codegen_scalar_type {
                self.scalar_extensions
                    .insert(type_name.clone(), codegen_scalar_type.to_string());
            }
        }
    }
    fn schema_addition(&self) -> Option<String> {
        let mut schema_addition = String::new();
        // Sort by type name to make the output deterministic.
        let mut scalar_extensions: Vec<_> = self.scalar_extensions.iter().collect();
        scalar_extensions.sort_by_key(|(type_name, _)| *type_name);
        for (type_name, codegen_scalar_type) in scalar_extensions {
            schema_addition.push_str(&format!(
                "extend scalar {type_name} @nitrogql_ts_type(
        resolverInput: \"{codegen_scalar_type}\"
        resolverOutput: \"{codegen_scalar_type}\"
        operationInput: \"{codegen_scalar_type}\"
        operationOutput: \"{codegen_scalar_type}\"
    )\n",
            ));
        }
        if schema_addition.is_empty() {
            None
        } else {
            Some(schema_addition)
        }
    }
}
