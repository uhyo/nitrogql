#![cfg(test)]

use std::collections::HashMap;

use crate::{GraphQLScalarsPlugin, PluginV1Beta};

use insta::assert_snapshot;

#[test]
fn schema_emission() {
    let mut p = GraphQLScalarsPlugin::default();
    let extensions: HashMap<String, HashMap<String, serde_yaml::Value>> = serde_yaml::from_str(
        r#"{
    "Date": {
        "nitrogql:kind": "scalar",
        "codegenScalarType": "Date"
    },
    "DateTime": {
        "nitrogql:kind": "scalar",
        "codegenScalarType": {
            "send": "Date | string",
            "receive": "string"
        }
    },
    "Time": {
        "nitrogql:kind": "scalar",
        "codegenScalarType": {
            "input": "string",
            "output": "string | number"
        }
    },
    "MysteryType": {
        "nitrogql:kind": "scalar",
        "codegenScalarType": {
            "resolverInput": "number",
            "resolverOutput": "string | number | bigint",
            "operationInput": "string",
            "operationOutput": "number | bigint"
        }
    },
    "Other": {
        "nitrogql:kind": "object",
        "codegenScalarType": "string"
    }
}"#,
    )
    .unwrap();
    p.load_schema_extensions(crate::PluginSchemaExtensions {
        type_extensions: &extensions,
    });
    assert_snapshot!(p.schema_addition().unwrap());
}
