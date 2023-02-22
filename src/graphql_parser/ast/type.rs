use super::base::{HasPos, Ident, Pos};

#[derive(Clone, Debug)]
pub enum Type<'a> {
    Named(NamedType<'a>),
    NonNull(Box<NonNullType<'a>>),
    List(Box<ListType<'a>>),
}

impl HasPos for Type<'_> {
    fn name(&self) -> Option<&str> {
        match self {
            Type::Named(name) => Some(name.name.name),
            Type::NonNull(_) => None,
            Type::List(_) => None,
        }
    }
    fn position(&self) -> &Pos {
        match self {
            Type::Named(name) => name.name.position(),
            Type::NonNull(non_null) => non_null.r#type.position(),
            Type::List(list) => &list.position,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NamedType<'a> {
    pub name: Ident<'a>,
}

#[derive(Clone, Debug)]
pub struct NonNullType<'a> {
    pub r#type: Type<'a>,
}

#[derive(Clone, Debug)]
pub struct ListType<'a> {
    pub position: Pos,
    pub r#type: Type<'a>,
}
