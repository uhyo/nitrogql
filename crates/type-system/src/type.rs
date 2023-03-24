use std::{fmt::Display, ops::Deref};

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
    /// Returns unwrapped type of this type.
    pub fn unwrapped(&self) -> &NamedType<Str, OriginalNode> {
        match self {
            Type::Named(named) => named,
            Type::List(inner) => inner.inner.unwrapped(),
            Type::NonNull(inner) => inner.inner.unwrapped(),
        }
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

impl<Str, OriginalNode> Deref for NamedType<Str, OriginalNode> {
    type Target = Node<Str, OriginalNode>;
    fn deref(&self) -> &Self::Target {
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

    pub fn as_inner(&self) -> &Type<Str, OriginalNode> {
        &self.inner
    }
}

impl<Str, OriginalNode> Deref for ListType<Str, OriginalNode> {
    type Target = Type<Str, OriginalNode>;
    fn deref(&self) -> &Self::Target {
        self.as_inner()
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

    pub fn as_inner(&self) -> &Type<Str, OriginalNode> {
        &self.inner
    }
}

impl<Str, OriginalNode> Deref for NonNullType<Str, OriginalNode> {
    type Target = Type<Str, OriginalNode>;
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}
