use std::collections::{HashMap, hash_map::Entry};

use crate::{DirectiveDefinition, Node, Schema, TypeDefinition, root_types::RootTypes, text::Text};

type SchemaRootTypes<Str, OriginalNode> = RootTypes<Option<Node<Str, OriginalNode>>>;

/// Struct for building Schema.
pub struct SchemaBuilder<Str, OriginalNode> {
    description: Option<Node<Str, OriginalNode>>,
    type_definitions: HashMap<Str, Node<TypeDefinition<Str, OriginalNode>, OriginalNode>>,
    directive_definitions: HashMap<Str, Node<DirectiveDefinition<Str, OriginalNode>, OriginalNode>>,
    type_names: Vec<Str>,
    directive_names: Vec<Str>,
    root_types: Option<Node<SchemaRootTypes<Str, OriginalNode>, OriginalNode>>,
}

impl<Str, OriginalNode: Default> SchemaBuilder<Str, OriginalNode> {
    /// Create an empty Schema.
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_description(&mut self, description: Node<Str, OriginalNode>) {
        self.description = Some(description);
    }
    pub fn set_root_types(
        &mut self,
        node: OriginalNode,
    ) -> &mut RootTypes<Option<Node<Str, OriginalNode>>> {
        match self.root_types {
            None => {
                self.root_types = Some(Node::from(RootTypes::default(), node));
                self.root_types.as_mut().unwrap()
            }
            Some(ref mut root_types) => root_types,
        }
    }
}
impl<Str, OriginalNode> Default for SchemaBuilder<Str, OriginalNode> {
    fn default() -> Self {
        Self {
            description: None,
            type_definitions: HashMap::new(),
            directive_definitions: HashMap::new(),
            type_names: vec![],
            directive_names: vec![],
            root_types: None,
        }
    }
}

impl<'a, Str: Text<'a>, OriginalNode>
    Extend<(Str, Node<TypeDefinition<Str, OriginalNode>, OriginalNode>)>
    for SchemaBuilder<Str, OriginalNode>
{
    fn extend<
        T: IntoIterator<Item = (Str, Node<TypeDefinition<Str, OriginalNode>, OriginalNode>)>,
    >(
        &mut self,
        iter: T,
    ) {
        for (key, def) in iter {
            let entry = self.type_definitions.entry(key);
            if matches!(entry, Entry::Vacant(_)) {
                self.type_names.push(entry.key().clone());
            }
            entry.or_insert(def);
        }
    }
}

impl<'a, Str: Text<'a>, OriginalNode>
    Extend<(
        Str,
        Node<DirectiveDefinition<Str, OriginalNode>, OriginalNode>,
    )> for SchemaBuilder<Str, OriginalNode>
{
    fn extend<
        T: IntoIterator<
            Item = (
                Str,
                Node<DirectiveDefinition<Str, OriginalNode>, OriginalNode>,
            ),
        >,
    >(
        &mut self,
        iter: T,
    ) {
        for (key, def) in iter {
            let entry = self.directive_definitions.entry(key);
            if matches!(entry, Entry::Vacant(_)) {
                self.directive_names.push(entry.key().clone());
            }
            entry.or_insert(def);
        }
    }
}

impl<Str, OriginalNode: Default> From<SchemaBuilder<Str, OriginalNode>>
    for Schema<Str, OriginalNode>
{
    fn from(value: SchemaBuilder<Str, OriginalNode>) -> Self {
        Schema {
            description: value.description,
            type_definitions: value.type_definitions,
            directive_definitions: value.directive_definitions,
            type_names: value.type_names,
            directive_names: value.directive_names,
            root_types: value
                .root_types
                .unwrap_or(Node::from(RootTypes::default(), OriginalNode::default())),
        }
    }
}
