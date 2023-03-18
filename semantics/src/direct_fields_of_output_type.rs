use nitrogql_ast::{
    base::{Ident, Pos},
    r#type::{NamedType, NonNullType, Type},
    type_system::{FieldDefinition, TypeDefinition},
};
use once_cell::sync::Lazy;

static TYPENAME_META_FIELD: Lazy<FieldDefinition<'static>> = Lazy::new(|| FieldDefinition {
    description: None,
    name: Ident {
        name: "__typename",
        position: Pos::builtin(),
    },
    arguments: None,
    r#type: Type::NonNull(Box::new(NonNullType {
        r#type: Type::Named(NamedType {
            name: Ident {
                name: "String",
                position: Pos::builtin(),
            },
        }),
    })),
    directives: vec![],
});

pub fn direct_fields_of_output_type<'a, 'src>(
    ty: &'a TypeDefinition<'src>,
) -> Option<Vec<&'a FieldDefinition<'src>>> {
    let meta_field: &FieldDefinition = &TYPENAME_META_FIELD;
    match ty {
        TypeDefinition::Object(obj) => Some(obj.fields.iter().chain(vec![meta_field]).collect()),
        TypeDefinition::Interface(obj) => Some(obj.fields.iter().chain(vec![meta_field]).collect()),
        TypeDefinition::Union(_) => Some(vec![meta_field]),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            None
        }
    }
}
