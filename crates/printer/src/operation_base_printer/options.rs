use nitrogql_config_file::Config;
use nitrogql_utils::clone_into;

/// Options for OperationBasePrinter.
#[derive(Debug, Clone)]
pub struct OperationBasePrinterOptions {
    /// Whether an operation should be default exported when it is the only operation in the document.
    pub default_export_for_operation: bool,
    /// Whether an operation should be named exported.
    pub named_export_for_operation: bool,
    /// Whether an operation input type should be exported.
    pub export_input_type: bool,
    /// Whether an operation result type should be exported.
    pub export_result_type: bool,
    /// Whether operation name should be capitalize
    pub capitalize_operation_names: bool,
    /// Suffix for variable of query.
    pub query_variable_suffix: String,
    /// Suffix for variable of mutation.
    pub mutation_variable_suffix: String,
    /// Suffix for variable of subscription.
    pub subscription_variable_suffix: String,
}

impl Default for OperationBasePrinterOptions {
    fn default() -> Self {
        Self {
            default_export_for_operation: true,
            named_export_for_operation: false,
            export_input_type: false,
            export_result_type: false,
            capitalize_operation_names: true,
            query_variable_suffix: "Query".to_owned(),
            mutation_variable_suffix: "Mutation".to_owned(),
            subscription_variable_suffix: "Subscription".to_owned(),
        }
    }
}

impl OperationBasePrinterOptions {
    /// Creates a new instance of OperationBasePrinterOptions from
    /// Config.
    pub fn from_config(config: &Config) -> Self {
        let mut result = Self {
            default_export_for_operation: config.generate.export.default_export_for_operation,
            export_input_type: config.generate.export.variables_type,
            export_result_type: config.generate.export.operation_result_type,
            ..Self::default()
        };
        clone_into(
            &config.generate.name.capitalize_operation_names,
            &mut result.capitalize_operation_names,
        );
        clone_into(
            &config.generate.name.query_variable_suffix,
            &mut result.query_variable_suffix,
        );
        clone_into(
            &config.generate.name.mutation_variable_suffix,
            &mut result.mutation_variable_suffix,
        );
        clone_into(
            &config.generate.name.subscription_variable_suffix,
            &mut result.subscription_variable_suffix,
        );
        result
    }
}
