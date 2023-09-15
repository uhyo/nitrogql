use crate::{cloning_utils::map_option_node, text::Text, Node};

#[derive(Debug, Copy, Clone)]
pub struct RootTypes<T> {
    /// Name of query root type.
    pub query_type: T,
    /// Name of mutation root type.
    pub mutation_type: T,
    /// Name of subscription root type.
    pub subscription_type: T,
}

impl<T> RootTypes<Option<T>> {
    pub fn set_query_type(&mut self, query_type: T) {
        self.query_type = Some(query_type);
    }
    pub fn set_mutation_type(&mut self, mutation_type: T) {
        self.mutation_type = Some(mutation_type);
    }
    pub fn set_subscription_type(&mut self, subscription_type: T) {
        self.subscription_type = Some(subscription_type);
    }
}

impl<T> Default for RootTypes<Option<T>> {
    fn default() -> Self {
        Self {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
        }
    }
}

impl<Str, OriginalNode> RootTypes<Option<Node<Str, OriginalNode>>>
where
    OriginalNode: Clone,
{
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> RootTypes<Option<Node<U, OriginalNode>>> {
        RootTypes {
            query_type: map_option_node(&self.query_type, &f),
            mutation_type: map_option_node(&self.mutation_type, &f),
            subscription_type: map_option_node(&self.subscription_type, &f),
        }
    }
}

impl<'a, Str: Text<'a>, OriginalNode: Clone + Default> RootTypes<Option<Node<Str, OriginalNode>>> {
    /// Unwrap root type names with default names.
    pub fn unwrap_or_default(&self) -> RootTypes<Node<Str, OriginalNode>> {
        RootTypes {
            query_type: self
                .query_type
                .clone()
                .unwrap_or(Node::from("Query", OriginalNode::default())),
            mutation_type: self
                .mutation_type
                .clone()
                .unwrap_or(Node::from("Mutation", OriginalNode::default())),
            subscription_type: self
                .subscription_type
                .clone()
                .unwrap_or(Node::from("Subscription", OriginalNode::default())),
        }
    }
}
