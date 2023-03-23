use std::collections::{hash_map::Entry, HashMap};

use crate::{root_types::RootTypes, text::Text, DirectiveDefinition, Node, Schema, TypeDefinition};

/// Struct for building Schema.
pub struct SchemaBuilder<Str, OriginalNode> {
    description: Option<Node<Str, OriginalNode>>,
    type_definitions: HashMap<Str, TypeDefinition<Str, OriginalNode>>,
    directive_definitions: HashMap<Str, DirectiveDefinition<Str, OriginalNode>>,
    type_names: Vec<Str>,
    directive_names: Vec<Str>,
    root_types: Option<Node<RootTypes<Option<Node<Str, OriginalNode>>>, OriginalNode>>,
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
                self.root_types.as_mut().unwrap().as_mut()
            }
            Some(ref mut root_types) => root_types.as_mut(),
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

impl<'a, Str: Text<'a>, OriginalNode> Extend<(Str, TypeDefinition<Str, OriginalNode>)>
    for SchemaBuilder<Str, OriginalNode>
{
    fn extend<T: IntoIterator<Item = (Str, TypeDefinition<Str, OriginalNode>)>>(
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

impl<'a, Str: Text<'a>, OriginalNode> Extend<(Str, DirectiveDefinition<Str, OriginalNode>)>
    for SchemaBuilder<Str, OriginalNode>
{
    fn extend<T: IntoIterator<Item = (Str, DirectiveDefinition<Str, OriginalNode>)>>(
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

impl<Str, OriginalNode: Default> Into<Schema<Str, OriginalNode>>
    for SchemaBuilder<Str, OriginalNode>
{
    fn into(self) -> Schema<Str, OriginalNode> {
        Schema {
            description: self.description,
            type_definitions: self.type_definitions,
            directive_definitions: self.directive_definitions,
            type_names: self.type_names,
            directive_names: self.directive_names,
            root_types: self
                .root_types
                .unwrap_or(Node::from(RootTypes::default(), OriginalNode::default())),
        }
    }
}

fn get_or_insert_default<T: Default>(option: &mut Option<T>) -> &mut T {
    match option {
        None => {
            option.insert(T::default());
            option.as_mut().unwrap()
        }
        Some(ref mut v) => v,
    }
}
