use std::fmt::Display;

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

impl<T, OriginalNode> AsRef<T> for Node<T, OriginalNode> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T, OriginalNode> AsMut<T> for Node<T, OriginalNode> {
    fn as_mut(&mut self) -> &mut T {
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

pub trait OriginalNodeRef<OriginalNode> {
    fn original_node_ref(&self) -> &OriginalNode;
}

impl<Str, OriginalNode> OriginalNodeRef<OriginalNode> for Node<Str, OriginalNode> {
    /// Returns a reference to original node.
    fn original_node_ref(&self) -> &OriginalNode {
        &self.original_node
    }
}
