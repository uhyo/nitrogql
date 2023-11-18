//! This module contains non-standard extension to the GraphQL syntax.

use crate::{
    base::{HasPos, Ident, Pos},
    operation::{FragmentDefinition, OperationDefinition},
    value::StringValue,
};

#[derive(Clone, Debug)]
pub struct OperationDocumentExt<'a> {
    pub definitions: Vec<ExecutableDefinitionExt<'a>>,
}

#[derive(Clone, Debug)]
pub enum ExecutableDefinitionExt<'a> {
    OperationDefinition(OperationDefinition<'a>),
    FragmentDefinition(FragmentDefinition<'a>),
    Import(ImportDefinition<'a>),
}

#[derive(Clone, Debug)]
pub struct ImportDefinition<'a> {
    pub position: Pos,
    pub targets: Vec<ImportTarget<'a>>,
    pub path: StringValue,
}

impl HasPos for ImportDefinition<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, Debug)]
pub enum ImportTarget<'a> {
    Wildcard,
    Name(Ident<'a>),
}
