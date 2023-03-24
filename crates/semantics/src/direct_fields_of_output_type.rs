use graphql_type_system::{Field, NamedType, Node, NonNullType, Type, TypeDefinition};
use nitrogql_ast::base::Pos;
use once_cell::sync::Lazy;

static TYPENAME_META_FIELD: Lazy<Field<&'static str, Pos>> = Lazy::new(|| Field {
    description: None,
    name: Node::from("__typename", Pos::builtin()),
    arguments: vec![],
    r#type: Type::NonNull(Box::new(NonNullType::from(Type::Named(NamedType::from(
        Node::from("String", Pos::builtin()),
    ))))),
});

pub fn direct_fields_of_output_type<'a, 'src>(
    ty: &'a TypeDefinition<&'src str, Pos>,
) -> Option<Vec<&'a Field<&'src str, Pos>>> {
    let meta_field: &Field<&str, Pos> = &TYPENAME_META_FIELD;
    match ty {
        TypeDefinition::Object(obj) => Some(obj.fields.iter().chain(vec![meta_field]).collect()),
        TypeDefinition::Interface(obj) => Some(obj.fields.iter().chain(vec![meta_field]).collect()),
        TypeDefinition::Union(_) => Some(vec![meta_field]),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            None
        }
    }
}
