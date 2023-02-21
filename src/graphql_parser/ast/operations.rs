use super::{
    base::{Pos, Variable},
    r#type::Type,
    value::Value,
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
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
}
