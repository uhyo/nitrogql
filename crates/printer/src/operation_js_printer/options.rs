use nitrogql_config_file::Config;

use crate::operation_base_printer::options::OperationBasePrinterOptions;

#[derive(Clone, Debug, Default)]
pub struct OperationJSPrinterOptions {
    pub base_options: OperationBasePrinterOptions,
}

impl OperationJSPrinterOptions {
    /// Generate from Config.
    pub fn from_config(config: &Config) -> Self {
        Self {
            base_options: OperationBasePrinterOptions::from_config(config),
        }
    }
}
