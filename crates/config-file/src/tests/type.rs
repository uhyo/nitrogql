use std::collections::HashMap;

use crate::parse_config;

#[test]
fn default_config() {
    let config = r#"
schema: schema.graphql
documents: []
"#;
    let config = parse_config(config).unwrap();
    let ty = config.generate.r#type;
    assert_eq!(ty.scalar_types, HashMap::new());
    assert!(ty.allow_undefined_as_optional_input);
}

#[test]
fn empty_object() {
    let config = r#"
schema: schema.graphql
documents: []
extensions:
    nitrogql:
        generate:
            type: {}
"#;
    let config = parse_config(config).unwrap();
    let ty = config.generate.r#type;
    assert_eq!(ty.scalar_types, HashMap::new());
    assert!(ty.allow_undefined_as_optional_input);
}

#[test]
fn scalar_types() {
    let config = r#"
schema: schema.graphql
documents: []
extensions:
    nitrogql:
        generate:
            type:
                scalarTypes:
                    DateTime: Date
                    JSON: any
                allowUndefinedAsInput: false
"#;
    let config = parse_config(config).unwrap();
    let ty = config.generate.r#type;
    let mut expected = HashMap::new();
    expected.insert("DateTime".to_string(), "Date".to_string());
    expected.insert("JSON".to_string(), "any".to_string());
    assert_eq!(ty.scalar_types, expected);
    assert!(!ty.allow_undefined_as_optional_input);
}
