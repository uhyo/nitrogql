use graphql_type_system::{ListType, NamedType, Node, NonNullType, Type};
use nitrogql_ast::{
    base::{Ident, Pos},
    r#type::Type as AstType,
};

/// Convert AST type to Type System type.
pub fn convert_type<'src>(ty: &AstType<'src>) -> Type<&'src str, Pos> {
    match ty {
        AstType::Named(ty) => Type::Named(NamedType::from(ident_to_node(&ty.name))),
        AstType::List(ty) => Type::List(Box::new(ListType::from(convert_type(&ty.r#type)))),
        AstType::NonNull(ty) => {
            Type::NonNull(Box::new(NonNullType::from(convert_type(&ty.r#type))))
        }
    }
}

pub fn ident_to_node<'src>(ident: &Ident<'src>) -> Node<&'src str, Pos> {
    Node::from(ident.name, ident.position)
}
