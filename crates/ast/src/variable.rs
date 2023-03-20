use crate::{
    base::{HasPos, Pos},
    directive::Directive,
    r#type::Type,
    value::Value,
};

/// Variable token.
#[derive(Copy, Clone, Debug)]
pub struct Variable<'a> {
    /// Variable name that does not include '$'
    pub name: &'a str,
    /// Position of '$'
    pub position: Pos,
}

impl HasPos for Variable<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.name)
    }
}

#[derive(Clone, Debug)]
pub struct VariablesDefinition<'a> {
    pub position: Pos,
    pub definitions: Vec<VariableDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct VariableDefinition<'a> {
    pub pos: Pos,
    pub name: Variable<'a>,
    pub r#type: Type<'a>,
    pub default_value: Option<Value<'a>>,
    pub directives: Vec<Directive<'a>>,
}
