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
    pub fn from(inner: T, original_node: OriginalNode) -> Self {
        Self {
            inner,
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
}

impl<T, OriginalNode> Deref for Node<T, OriginalNode> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
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

impl<T: PartialEq, OriginalNode> PartialEq<T> for Node<T, OriginalNode> {
    fn eq(&self, other: &T) -> bool {
        self.inner == *other
    }
}

// Note: equality between Node does not take OriginalNode in consideration.
impl<T: PartialEq, OriginalNode, OriginalNode2> PartialEq<Node<T, OriginalNode2>>
    for Node<T, OriginalNode>
{
    fn eq(&self, other: &Node<T, OriginalNode2>) -> bool {
        self.inner == other.inner
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
