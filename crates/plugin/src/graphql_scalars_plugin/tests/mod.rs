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
        "codegenScalarType": "number"
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
