use std::path::PathBuf;

use crate::{parse_config, GenerateMode};

#[test]
fn parse_schema_and_documents() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
"#;
    let config = parse_config(config).unwrap();
    assert_eq!(config.schema, vec!["schema.graphql"]);
    assert_eq!(config.operations, vec!["src/**/*.graphql"]);
}

#[test]
fn parse_schema_and_documents_as_array() {
    let config = r#"
schema:
    - schema.graphql
documents:
    - src/**/*.graphql
    - src/**/*.graphqls
"#;
    let config = parse_config(config).unwrap();
    assert_eq!(config.schema, vec!["schema.graphql"]);
    assert_eq!(
        config.operations,
        vec!["src/**/*.graphql", "src/**/*.graphqls"]
    );
}

#[test]
fn parse_generate_config() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
extensions:
    nitrogql:
        generate:
            mode: with-loader-ts-5.0
            schemaOutput: src/generated/schema.d.ts
            schemaModuleSpecifier: "@generated/schema"
"#;

    let config = parse_config(config).unwrap();
    assert_eq!(config.generate.mode, GenerateMode::WithLoaderTS5_0);
    assert_eq!(
        config.generate.schema_output,
        Some(PathBuf::from("src/generated/schema.d.ts"))
    );
    assert_eq!(
        config.generate.schema_module_specifier,
        Some("@generated/schema".to_owned())
    );
}

#[test]
fn parse_scalar_types() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
extensions:
    nitrogql:
        generate:
            scalarTypes:
                Date: Date
                BigInt: bigint
                Int: number
                Float: number
                ID: string
                String: string
"#;

    let config = parse_config(config).unwrap();
    assert_eq!(
        config.generate.scalar_types,
        vec![
            ("Date".to_owned(), "Date".to_owned()),
            ("BigInt".to_owned(), "bigint".to_owned()),
            ("Int".to_owned(), "number".to_owned()),
            ("Float".to_owned(), "number".to_owned()),
            ("ID".to_owned(), "string".to_owned()),
            ("String".to_owned(), "string".to_owned()),
        ]
        .into_iter()
        .collect()
    );
}

#[test]
fn json_is_yaml() {
    let config = r#"
{
    "schema": "schema.graphql",
    "documents": "src/**/*.graphql",
    "extensions": {
        "nitrogql": {
            "generate": {
                "mode": "with-loader-ts-5.0",
                "schemaOutput": "src/generated/schema.d.ts",
                "schemaModuleSpecifier": "@generated/schema"
            }
        }
    }
}"#;

    let config = parse_config(config).unwrap();
    assert_eq!(config.schema, vec!["schema.graphql"]);
    assert_eq!(config.operations, vec!["src/**/*.graphql"]);
    assert_eq!(config.generate.mode, GenerateMode::WithLoaderTS5_0);
    assert_eq!(
        config.generate.schema_output,
        Some(PathBuf::from("src/generated/schema.d.ts"))
    );
    assert_eq!(
        config.generate.schema_module_specifier,
        Some("@generated/schema".to_owned())
    );
}

#[test]
fn extra_fields_are_ignored() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
extensions:
    nitrogql:
        generate:
            mode: standalone-ts-4.0
            schemaOutput: src/generated/schema.d.ts
            schemaModuleSpecifier: "@generated/schema"
            extra: "field"
"#;

    let config = parse_config(config).unwrap();
    assert_eq!(config.generate.mode, GenerateMode::StandaloneTS4_0);
    assert_eq!(
        config.generate.schema_output,
        Some(PathBuf::from("src/generated/schema.d.ts"))
    );
    assert_eq!(
        config.generate.schema_module_specifier,
        Some("@generated/schema".to_owned())
    );
}
