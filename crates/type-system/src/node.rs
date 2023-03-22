use std::borrow::Borrow;

/// Object that might be associated with an original node.
#[derive(Copy, Clone, Debug)]
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
    /// Returns a reference to original node.
    pub fn original_node_ref(&self) -> &OriginalNode {
        &self.original_node
    }
}

impl<T, OriginalNode> AsRef<T> for Node<T, OriginalNode> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}
