use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

/// Object that might be associated with an original node.
#[derive(Copy, Clone, Debug, Default)]
pub struct Node<T, OriginalNode> {
    inner: T,
    original_node: OriginalNode,
}

impl<T, OriginalNode> Node<T, OriginalNode> {
    /// Creates a new Node.
    pub fn from(inner: impl Into<T>, original_node: OriginalNode) -> Self {
        Self {
            inner: inner.into(),
            original_node,
        }
    }
    /// Returns content of self.
    pub fn into_inner(self) -> T {
        self.inner
    }
    /// Returns node associated to self.
    pub fn into_original_node(self) -> OriginalNode {
        self.original_node
    }
    /// Returns a reference to the inner value.
    pub fn inner_ref(&self) -> &T {
        &self.inner
    }
}

impl<T, OriginalNode> Deref for Node<T, OriginalNode> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner_ref()
    }
}

impl<T, OriginalNode> DerefMut for Node<T, OriginalNode> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Display, OriginalNode> Display for Node<T, OriginalNode> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

// Note: equality between Node does not take OriginalNode in consideration.
impl<Other, T: PartialEq<Other>, OriginalNode> PartialEq<Other> for Node<T, OriginalNode> {
    fn eq(&self, other: &Other) -> bool {
        self.inner == *other
    }
}

pub trait OriginalNodeRef<OriginalNode> {
    fn original_node_ref(&self) -> &OriginalNode;
}

impl<Str, OriginalNode> OriginalNodeRef<OriginalNode> for Node<Str, OriginalNode> {
    /// Returns a reference to original node.
    fn original_node_ref(&self) -> &OriginalNode {
        &self.original_node
    }
}
