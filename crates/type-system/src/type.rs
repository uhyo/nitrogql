use std::fmt::Display;

use crate::node::Node;

/// Represents a type.
#[derive(Debug, Clone)]
pub enum Type<Str, OriginalNode> {
    Named(NamedType<Str, OriginalNode>),
    List(Box<ListType<Str, OriginalNode>>),
    NonNull(Box<NonNullType<Str, OriginalNode>>),
}

impl<Str, OriginalNode> Type<Str, OriginalNode> {
    /// Returns whether this is a non-null type.
    pub fn is_nonnull(&self) -> bool {
        matches!(self, Type::NonNull(_))
    }
}

impl<Str: Display, OriginalNode> Display for Type<Str, OriginalNode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Named(inner) => write!(f, "{}", inner.name),
            Type::List(inner) => write!(f, "[{}]", inner.inner),
            Type::NonNull(inner) => write!(f, "{}!", inner.inner),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamedType<Str, OriginalNode> {
    name: Node<Str, OriginalNode>,
}

impl<Str, OriginalNode> AsRef<Node<Str, OriginalNode>> for NamedType<Str, OriginalNode> {
    fn as_ref(&self) -> &Node<Str, OriginalNode> {
        &self.name
    }
}

impl<Str, OriginalNode> NamedType<Str, OriginalNode> {
    pub fn from(name: Node<Str, OriginalNode>) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone)]
pub struct ListType<Str, OriginalNode> {
    inner: Type<Str, OriginalNode>,
}

impl<Str, OriginalNode> ListType<Str, OriginalNode> {
    pub fn from(inner: Type<Str, OriginalNode>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> Type<Str, OriginalNode> {
        self.inner
    }
}

impl<Str, OriginalNode> AsRef<Type<Str, OriginalNode>> for ListType<Str, OriginalNode> {
    fn as_ref(&self) -> &Type<Str, OriginalNode> {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct NonNullType<Str, OriginalNode> {
    inner: Type<Str, OriginalNode>,
}

impl<Str, OriginalNode> NonNullType<Str, OriginalNode> {
    pub fn from(inner: Type<Str, OriginalNode>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> Type<Str, OriginalNode> {
        self.inner
    }
}

impl<Str, OriginalNode> AsRef<Type<Str, OriginalNode>> for NonNullType<Str, OriginalNode> {
    fn as_ref(&self) -> &Type<Str, OriginalNode> {
        &self.inner
    }
}
