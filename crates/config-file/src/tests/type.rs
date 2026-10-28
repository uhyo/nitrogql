use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{ScalarTypeConfig, parse_config};

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
                allowUndefinedAsOptionalInput: false
"#;
    let config = parse_config(config).unwrap();
    let ty = config.generate.r#type;
    let mut expected = HashMap::new();
    expected.insert(
        "DateTime".to_string(),
        ScalarTypeConfig::Single("Date".to_string()),
    );
    expected.insert(
        "JSON".to_string(),
        ScalarTypeConfig::Single("any".to_string()),
    );
    assert_eq!(ty.scalar_types, expected);
    assert!(!ty.allow_undefined_as_optional_input);
}

#[test]
fn equality_and_hash_ignore_order_of_scalar_types() {
    fn hash(config: &crate::Config) -> u64 {
        let mut hasher = DefaultHasher::new();
        config.hash(&mut hasher);
        hasher.finish()
    }
    let names = ["A", "B", "C", "D", "E", "F", "G", "H"];
    let scalar_types = |names: &[&str]| {
        names
            .iter()
            .map(|name| format!("                {name}: {name}Type\n"))
            .collect::<String>()
    };
    let config = |scalar_types: String| {
        parse_config(&format!(
            r#"
schema: schema.graphql
documents: []
extensions:
    nitrogql:
        generate:
            type:
              scalarTypes:
{scalar_types}"#
        ))
        .unwrap()
    };
    let reversed = names.iter().rev().copied().collect::<Vec<_>>();
    let config1 = config(scalar_types(&names));
    let config2 = config(scalar_types(&reversed));
    assert_eq!(config1, config2);
    assert_eq!(hash(&config1), hash(&config2));

    let config3 = config(scalar_types(&names[1..]));
    assert_ne!(config1, config3);
    assert_ne!(hash(&config1), hash(&config3));
}
