use std::collections::HashSet;

use super::{ts_types_util::ts_intersection, ObjectField, TSType};

/// Merges given list of object fields into a single object type.
/// If a field with the same name already exists, they are deeply merged into a single field.
pub fn deep_merge_to_object(fields: impl IntoIterator<Item = ObjectField>) -> TSType {
    let mut seen_fields = HashSet::<String>::new();
    let mut new_fields = Vec::<ObjectField>::new();
    for field in fields {
        if seen_fields.contains(&field.key.name) {
            let existing = new_fields
                .iter_mut()
                .find(|f| f.key.name == field.key.name)
                .expect("field was just inserted");
            let existing_type = std::mem::replace(&mut existing.r#type, TSType::Null);
            let new_type = ts_intersection(vec![existing_type, field.r#type]);
            existing.r#type = new_type;
        } else {
            seen_fields.insert(field.key.name.clone());
            new_fields.push(field);
        }
    }
    TSType::Object(new_fields)
}
