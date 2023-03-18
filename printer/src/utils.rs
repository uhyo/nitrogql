use nitrogql_ast::type_system::{
    ObjectTypeDefinition, TypeDefinition, TypeSystemDefinition, TypeSystemDocument,
};

/// Returns an iterator over possible object types that implements given interface.
pub fn interface_implementers<'a>(
    schema: &'a TypeSystemDocument<'a>,
    interface_name: &'a str,
) -> impl Iterator<Item = &'a ObjectTypeDefinition<'a>> + 'a {
    schema.definitions.iter().filter_map(move |def| {
        if let TypeSystemDefinition::TypeDefinition(TypeDefinition::Object(ref obj_def)) = def {
            if obj_def
                .implements
                .iter()
                .any(|imp| imp.name == interface_name)
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
