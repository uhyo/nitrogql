use crate::parse_config;

#[test]
fn default_config() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
"#;
    let config = parse_config(config).unwrap();
    let ex = config.generate.export;
    assert!(ex.default_export_for_operation);
    assert!(!ex.operation_result_type);
    assert!(!ex.variables_type);
}

#[test]
fn empty_object() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
extensions:
    nitrogql:
        generate:
            export: {}
"#;
    let config = parse_config(config).unwrap();
    let ex = config.generate.export;
    assert!(ex.default_export_for_operation);
    assert!(!ex.operation_result_type);
    assert!(!ex.variables_type);
}

#[test]
fn export_config() {
    let config = r#"
schema: schema.graphql
documents: src/**/*.graphql
extensions:
    nitrogql:
        generate:
            export:
                defaultExportForOperation: false
                operationResultType: true
                variablesType: true
"#;
    let config = parse_config(config).unwrap();
    let ex = config.generate.export;
    assert!(!ex.default_export_for_operation);
    assert!(ex.operation_result_type);
    assert!(ex.variables_type);
}
