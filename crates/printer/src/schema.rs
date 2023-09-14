use std::collections::HashMap;

/// Generates scalar definitions for built-in scalars.
pub fn get_builtin_scalar_types() -> HashMap<String, String> {
    vec![
        ("ID".into(), "string".into()),
        ("String".into(), "string".into()),
        ("Int".into(), "number".into()),
        ("Float".into(), "number".into()),
        ("Boolean".into(), "boolean".into()),
    ]
    .into_iter()
    .collect()
}
