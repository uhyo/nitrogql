use crate::variable::VariablesDefinition;

use super::{
    base::{HasPos, Ident, NamePos, Pos},
    directive::Directive,
    selection_set::SelectionSet,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
            OperationType::Subscription => "subscription",
        }
    }
}

#[derive(Clone, Debug)]
pub enum ExecutableDefinition<'a> {
    OperationDefinition(OperationDefinition<'a>),
    FragmentDefinition(FragmentDefinition<'a>),
}

impl HasPos for ExecutableDefinition<'_> {
    fn name(&self) -> Option<&str> {
        match self {
            ExecutableDefinition::OperationDefinition(def) => def.name(),
            ExecutableDefinition::FragmentDefinition(def) => def.name(),
        }
    }
    fn position(&self) -> &Pos {
        match self {
            ExecutableDefinition::OperationDefinition(def) => def.position(),
            ExecutableDefinition::FragmentDefinition(def) => def.position(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct OperationDefinition<'a> {
    pub position: Pos,
    pub operation_type: OperationType,
    pub name: Option<Ident<'a>>,
    pub variables_definition: Option<VariablesDefinition<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

impl HasPos for OperationDefinition<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        self.name.map(|name| name.name)
    }
}

impl OperationDefinition<'_> {
    /// Returns Pos for its name.
    pub fn name_pos(&self) -> NamePos {
        match self.name {
            None => NamePos {
                pos: *self.position(),
                name: None,
            },
            Some(ref name) => NamePos {
                pos: *name.position(),
                name: Some(name.name),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct FragmentDefinition<'a> {
    pub position: Pos,
    pub name: Ident<'a>,
    pub type_condition: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}

impl HasPos for FragmentDefinition<'_> {
    fn name(&self) -> Option<&str> {
        Some(self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    /// Position of document. This is the position of the first character of the document.
    /// Mainly useful for knowing the file index.
    pub position: Pos,
    pub definitions: Vec<ExecutableDefinition<'a>>,
}
