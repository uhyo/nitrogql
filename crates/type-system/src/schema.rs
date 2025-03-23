use std::{collections::HashMap, hash::Hash};

use crate::{
    cloning_utils::map_option_node,
    definitions::{DirectiveDefinition, TypeDefinition},
    node::Node,
    root_types::RootTypes,
    text::Text,
};

/// Representation of GraphQL Type System.
#[derive(Debug, Clone)]
pub struct Schema<Str, OriginalNode> {
    /// Description of schema.
    pub(crate) description: Option<Node<Str, OriginalNode>>,
    /// Types in this schema.
    pub(crate) type_definitions:
        HashMap<Str, Node<TypeDefinition<Str, OriginalNode>, OriginalNode>>,
    /// Directives in this schema.
    pub(crate) directive_definitions:
        HashMap<Str, Node<DirectiveDefinition<Str, OriginalNode>, OriginalNode>>,
    /// Keeps insertion order for stable iteration order.
    pub(crate) type_names: Vec<Str>,
    /// Keeps insertion order for stable iteration order.
    pub(crate) directive_names: Vec<Str>,
    pub(crate) root_types: Node<RootTypes<Option<Node<Str, OriginalNode>>>, OriginalNode>,
}

impl<Str, OriginalNode> Schema<Str, OriginalNode> {
    /// Returns description of schema.
    pub fn description(&self) -> &Option<Node<Str, OriginalNode>> {
        &self.description
    }
    /// Returns the set of root operation types.
    pub fn root_types(&self) -> &Node<RootTypes<Option<Node<Str, OriginalNode>>>, OriginalNode> {
        &self.root_types
    }
}

impl<Str, OriginalNode> Schema<Str, OriginalNode>
where
    OriginalNode: Clone,
{
    /// Maps all string values in this schema.
    pub fn map_str<U>(&self, f: impl Fn(&Str) -> U) -> Schema<U, OriginalNode>
    where
        U: Eq + Hash,
    {
        Schema {
            description: map_option_node(&self.description, &f),
            type_definitions: self
                .type_definitions
                .iter()
                .map(|(k, v)| (f(k), v.as_ref().map(|x| x.map_str(&f))))
                .collect(),
            directive_definitions: self
                .directive_definitions
                .iter()
                .map(|(k, v)| (f(k), v.as_ref().map(|x| x.map_str(&f))))
                .collect(),
            type_names: self.type_names.iter().map(&f).collect(),
            directive_names: self.directive_names.iter().map(&f).collect(),
            root_types: self
                .root_types
                .as_ref()
                .map(|root_types| root_types.map_str(&f)),
        }
    }
}

impl<'a, Str: Text<'a>, OriginalNode> Schema<Str, OriginalNode> {
    /// Queries a type by name.
    pub fn get_type(
        &self,
        name: &str,
    ) -> Option<&Node<TypeDefinition<Str, OriginalNode>, OriginalNode>> {
        self.type_definitions.get(name)
    }
    /// Queries a directive by name.
    pub fn get_directive(
        &self,
        name: &str,
    ) -> Option<&Node<DirectiveDefinition<Str, OriginalNode>, OriginalNode>> {
        self.directive_definitions.get(name)
    }

    /// Iterate over types.
    pub fn iter_types(
        &self,
    ) -> impl Iterator<Item = (&Str, &Node<TypeDefinition<Str, OriginalNode>, OriginalNode>)> + use<'_, Str, OriginalNode> {
        self.type_names.iter().filter_map(move |type_name| {
            self.type_definitions
                .get(type_name.borrow())
                .map(|ty| (type_name, ty))
        })
    }
    /// Iterate over directives.
    pub fn iter_directives(
        &self,
    ) -> impl Iterator<
        Item = (
            &Str,
            &Node<DirectiveDefinition<Str, OriginalNode>, OriginalNode>,
        ),
    > + use<'_, Str, OriginalNode> {
        self.directive_names.iter().filter_map(move |type_name| {
            self.directive_definitions
                .get(type_name.borrow())
                .map(|ty| (type_name, ty))
        })
    }
}
