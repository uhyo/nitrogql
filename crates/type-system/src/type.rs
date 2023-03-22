use crate::{node::Node, text::Text};

/// Represents a type.
#[derive(Debug, Clone)]
pub enum Type<Str, OriginalNode> {
    Named(NamedType<Str, OriginalNode>),
    List(Box<ListType<Str, OriginalNode>>),
    NonNull(Box<NonNullType<Str, OriginalNode>>),
}

#[derive(Debug, Clone)]
pub struct NamedType<Str, OriginalNode> {
    name: Node<Str, OriginalNode>,
}

impl<Str: Text, OriginalNode> NamedType<Str, OriginalNode> {
    pub fn from(name: Node<Str, OriginalNode>) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone)]
pub struct ListType<Str, OriginalNode> {
    inner: Type<Str, OriginalNode>,
}

impl<Str: Text, OriginalNode> ListType<Str, OriginalNode> {
    pub fn from(inner: Type<Str, OriginalNode>) -> Self {
        Self { inner }
    }
}

#[derive(Debug, Clone)]
pub struct NonNullType<Str, OriginalNode> {
    inner: Type<Str, OriginalNode>,
}

impl<Str: Text, OriginalNode> NonNullType<Str, OriginalNode> {
    pub fn from(inner: Type<Str, OriginalNode>) -> Self {
        Self { inner }
    }
}
