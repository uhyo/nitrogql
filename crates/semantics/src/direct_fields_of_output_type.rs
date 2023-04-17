use std::borrow::Cow;

use graphql_type_system::{Field, NamedType, Node, NonNullType, Type, TypeDefinition};
use nitrogql_ast::base::Pos;

fn get_typename_meta_field<'a, S: From<&'a str>, D: Default>() -> Field<S, D> {
    Field {
        description: None,
        name: Node::from("__typename", D::default()),
        arguments: vec![],
        r#type: Type::NonNull(Box::new(NonNullType::from(Type::Named(NamedType::from(
            Node::from("String", D::default()),
        ))))),
        deprecation: None,
    }
}

pub fn direct_fields_of_output_type<'a, 'b, S: From<&'a str> + Clone>(
    ty: &'b TypeDefinition<S, Pos>,
) -> Option<Vec<Cow<'b, Field<S, Pos>>>> {
    let meta_field: Field<S, Pos> = get_typename_meta_field();
    match ty {
        TypeDefinition::Object(obj) => Some(
            obj.fields
                .iter()
                .map(Cow::Borrowed)
                .chain(vec![Cow::Owned(meta_field)])
                .collect(),
        ),
        TypeDefinition::Interface(obj) => Some(
            obj.fields
                .iter()
                .map(Cow::Borrowed)
                .chain(vec![Cow::Owned(meta_field)])
                .collect(),
        ),
        TypeDefinition::Union(_) => Some(vec![Cow::Owned(meta_field)]),
        TypeDefinition::Scalar(_) | TypeDefinition::Enum(_) | TypeDefinition::InputObject(_) => {
            None
        }
    }
}
