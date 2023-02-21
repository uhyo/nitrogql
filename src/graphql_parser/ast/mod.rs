use self::operations::ExecutableDefinition;

pub mod base;
pub mod directive;
pub mod operations;
pub mod selection_set;
pub mod r#type;
pub mod value;

#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    pub definitions: Vec<ExecutableDefinition<'a>>,
}
