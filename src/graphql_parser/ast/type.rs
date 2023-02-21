use super::base::{Ident, Pos};

#[derive(Clone, Debug)]
pub enum Type<'a> {
    Named(NamedType<'a>),
    NonNull(Box<NonNullType<'a>>),
    List(Box<ListType<'a>>),
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
