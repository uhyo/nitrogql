use std::collections::HashMap;

use nitrogql_config_file::ScalarTypeConfig;

/// Generates scalar definitions for built-in scalars.
pub fn get_builtin_scalar_types() -> HashMap<String, ScalarTypeConfig> {
    vec![
        ("ID".into(), ScalarTypeConfig::Single("string".into())),
        ("String".into(), ScalarTypeConfig::Single("string".into())),
        ("Int".into(), ScalarTypeConfig::Single("number".into())),
        ("Float".into(), ScalarTypeConfig::Single("number".into())),
        ("Boolean".into(), ScalarTypeConfig::Single("boolean".into())),
    ]
    .into_iter()
    .collect()
}
