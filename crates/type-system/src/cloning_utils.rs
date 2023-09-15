use crate::Node;

pub fn map_option_node<Str, OriginalNode, U>(
    node: &Option<Node<Str, OriginalNode>>,
    f: impl FnOnce(&Str) -> U,
) -> Option<Node<U, OriginalNode>>
where
    OriginalNode: Clone,
{
    node.as_ref().map(|node| node.as_ref().map(f))
}

pub fn map_vec_node<Str, OriginalNode, U>(
    slice: &[Node<Str, OriginalNode>],
    f: impl Fn(&Str) -> U,
) -> Vec<Node<U, OriginalNode>>
where
    OriginalNode: Clone,
{
    slice.iter().map(|node| node.as_ref().map(&f)).collect()
}
