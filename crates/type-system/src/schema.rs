use std::collections::HashMap;

use crate::{
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
    ) -> impl Iterator<Item = (&Str, &Node<TypeDefinition<Str, OriginalNode>, OriginalNode>)> {
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
    > {
        self.directive_names.iter().filter_map(move |type_name| {
            self.directive_definitions
                .get(type_name.borrow())
                .map(|ty| (type_name, ty))
        })
    }
}
