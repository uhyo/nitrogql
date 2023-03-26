use graphql_type_system::{ObjectDefinition, Schema, Text};

/// Returns an iterator over possible object types that implements given interface.
pub fn interface_implementers<'a, 'src, S: Text<'src>, OriginalNode>(
    schema: &'a Schema<S, OriginalNode>,
    interface_name: &'a str,
) -> impl Iterator<Item = &'a ObjectDefinition<S, OriginalNode>> + 'a {
    schema.iter_types().filter_map(move |(_, def)| {
        if let Some(obj_def) = def.as_object() {
            if obj_def
                .interfaces
                .iter()
                .any(|imp| imp.inner_ref().borrow() == interface_name)
            {
                Some(obj_def)
            } else {
                None
            }
        } else {
            None
        }
    })
}
