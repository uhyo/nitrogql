use crate::parse_config;

#[test]
fn default_config() {
    let config = r#"
schema: schema.graphql
"#;
    let config = parse_config(config).unwrap();
    let name = config.generate.name;
    assert_eq!(name.operation_result_type_suffix, None);
    assert_eq!(name.variables_type_suffix, None);
    assert_eq!(name.capitalize_operation_names, None);
    assert_eq!(name.query_variable_suffix, None);
    assert_eq!(name.mutation_variable_suffix, None);
    assert_eq!(name.subscription_variable_suffix, None);
}

#[test]
fn empty_object() {
    let config = r#"
schema: schema.graphql
extensions:
    nitrogql:
        generate:
            name: {}
"#;
    let config = parse_config(config).unwrap();
    let name = config.generate.name;
    assert_eq!(name.operation_result_type_suffix, None);
    assert_eq!(name.variables_type_suffix, None);
    assert_eq!(name.capitalize_operation_names, None);
    assert_eq!(name.query_variable_suffix, None);
    assert_eq!(name.mutation_variable_suffix, None);
    assert_eq!(name.subscription_variable_suffix, None);
}

#[test]
fn name_config() {
    let config = r#"
schema: schema.graphql
extensions:
    nitrogql:
        generate:
            name:
                operationResultTypeSuffix: Result
                variablesTypeSuffix: Variables
                capitalizeOperationNames: true
                queryVariableSuffix: Query
                mutationVariableSuffix: Mutation
                subscriptionVariableSuffix: Subscription
"#;
    let config = parse_config(config).unwrap();
    let name = config.generate.name;
    assert_eq!(
        name.operation_result_type_suffix,
        Some("Result".to_string())
    );
    assert_eq!(name.variables_type_suffix, Some("Variables".to_string()));
    assert_eq!(name.capitalize_operation_names, Some(true));
    assert_eq!(name.query_variable_suffix, Some("Query".to_string()));
    assert_eq!(name.mutation_variable_suffix, Some("Mutation".to_string()));
    assert_eq!(
        name.subscription_variable_suffix,
        Some("Subscription".to_string())
    );
}
