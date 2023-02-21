use self::{
    base::Ident,
    directive::Directive,
    operations::{OperationType, VariablesDefinition},
};

pub mod base;
pub mod directive;
pub mod operations;
pub mod r#type;
pub mod value;

#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    pub definitions: Vec<OperationDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct OperationDefinition<'a> {
    pub operation_type: OperationType,
    pub name: Option<Ident<'a>>,
    pub variables_definition: Option<VariablesDefinition<'a>>,
    pub directives: Vec<Directive<'a>>,
}
