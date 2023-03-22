use std::collections::{hash_map::Entry, HashMap};

use crate::{
    definitions::{DirectiveDefinition, TypeDefinition},
    node::Node,
    text::Text,
};

/// Representation of GraphQL Type System.
#[derive(Debug, Clone)]
pub struct Schema<Str, OriginalNode> {
    /// Description of schema.
    pub description: Option<Node<Str, OriginalNode>>,
    /// Types in this schema.
    type_definitions: HashMap<Str, TypeDefinition<Str, OriginalNode>>,
    /// Directives in this schema.
    directive_definitions: HashMap<Str, DirectiveDefinition<Str, OriginalNode>>,
    /// Keeps insertion order for stable iteration order.
    type_names: Vec<Str>,
    /// Keeps insertion order for stable iteration order.
    directive_names: Vec<Str>,
    /// Name of query root type.
    query_type: Option<Node<Str, OriginalNode>>,
    /// Name of mutation root type.
    mutation_type: Option<Node<Str, OriginalNode>>,
    /// Name of subscription root type.
    subscription_type: Option<Node<Str, OriginalNode>>,
}

/// Struct for building Schema.
pub struct SchemaBuilder<Str, OriginalNode> {
    schema: Schema<Str, OriginalNode>,
}

impl<Str, OriginalNode> SchemaBuilder<Str, OriginalNode> {
    /// Create an empty Schema.
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_description(&mut self, description: Node<Str, OriginalNode>) {
        self.schema.description = Some(description);
    }
    pub fn set_root_query_type(&mut self, query_type: Node<Str, OriginalNode>) {
        self.schema.query_type = Some(query_type);
    }
    pub fn set_root_mutation_type(&mut self, mutation_type: Node<Str, OriginalNode>) {
        self.schema.mutation_type = Some(mutation_type);
    }
    pub fn set_root_subscription_type(&mut self, subscription_type: Node<Str, OriginalNode>) {
        self.schema.subscription_type = Some(subscription_type);
    }
}
impl<Str: Text, OriginalNode> Schema<Str, OriginalNode> {
    /// Queries a type by name.
    pub fn get_type(&self, name: &str) -> Option<&TypeDefinition<Str, OriginalNode>> {
        self.type_definitions.get(name)
    }
    /// Queries a directive by name.
    pub fn get_directive(&self, name: &str) -> Option<&DirectiveDefinition<Str, OriginalNode>> {
        self.directive_definitions.get(name)
    }

    /// Iterate over types.
    pub fn iter_types<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Str, &'a TypeDefinition<Str, OriginalNode>)> {
        self.type_names.iter().filter_map(|type_name| {
            self.type_definitions
                .get(type_name.borrow())
                .map(|ty| (type_name, ty))
        })
    }
    /// Iterate over directives.
    pub fn iter_directives<'a>(
        &'a self,
    ) -> impl Iterator<Item = (&'a Str, &'a DirectiveDefinition<Str, OriginalNode>)> {
        self.directive_names.iter().filter_map(|type_name| {
            self.directive_definitions
                .get(type_name.borrow())
                .map(|ty| (type_name, ty))
        })
    }
}

impl<Str, OriginalNode> Default for SchemaBuilder<Str, OriginalNode> {
    fn default() -> Self {
        Self {
            schema: Schema {
                description: None,
                type_definitions: HashMap::new(),
                directive_definitions: HashMap::new(),
                type_names: vec![],
                directive_names: vec![],
                query_type: None,
                mutation_type: None,
                subscription_type: None,
            },
        }
    }
}

impl<Str: Text, OriginalNode> Extend<(Str, TypeDefinition<Str, OriginalNode>)>
    for SchemaBuilder<Str, OriginalNode>
{
    fn extend<T: IntoIterator<Item = (Str, TypeDefinition<Str, OriginalNode>)>>(
        &mut self,
        iter: T,
    ) {
        for (key, def) in iter {
            let entry = self.schema.type_definitions.entry(key);
            if matches!(entry, Entry::Vacant(_)) {
                self.schema.type_names.push(entry.key().clone());
            }
            entry.or_insert(def);
        }
    }
}

impl<Str: Text, OriginalNode> Extend<(Str, DirectiveDefinition<Str, OriginalNode>)>
    for SchemaBuilder<Str, OriginalNode>
{
    fn extend<T: IntoIterator<Item = (Str, DirectiveDefinition<Str, OriginalNode>)>>(
        &mut self,
        iter: T,
    ) {
        for (key, def) in iter {
            let entry = self.schema.directive_definitions.entry(key);
            if matches!(entry, Entry::Vacant(_)) {
                self.schema.directive_names.push(entry.key().clone());
            }
            entry.or_insert(def);
        }
    }
}

impl<Str, OriginalNode> Into<Schema<Str, OriginalNode>> for SchemaBuilder<Str, OriginalNode> {
    fn into(self) -> Schema<Str, OriginalNode> {
        self.schema
    }
}
