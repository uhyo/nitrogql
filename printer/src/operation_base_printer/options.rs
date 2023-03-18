/// Options for OperationBasePrinter.
#[derive(Debug, Clone)]
pub struct OperationBasePrinterOptions {
    /// Whether an operation should be default exported when it is the only operation in the document.
    pub default_export_for_operation: bool,
    /// Whether an operation should be named exported.
    pub named_export_for_operation: bool,
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
            capitalize_operation_names: true,
            query_variable_suffix: "Query".to_owned(),
            mutation_variable_suffix: "Mutation".to_owned(),
            subscription_variable_suffix: "Subscription".to_owned(),
        }
    }
}
