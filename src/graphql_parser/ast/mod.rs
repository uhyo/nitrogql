use self::{operations::ExecutableDefinition, type_system::TypeSystemDefinitionOrExtension};

pub mod base;
pub mod directive;
pub mod operations;
pub mod selection_set;
pub mod r#type;
pub mod type_system;
pub mod value;

#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    pub definitions: Vec<ExecutableDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct TypeSystemOrExtensionDocument<'a> {
    pub definitions: Vec<TypeSystemDefinitionOrExtension<'a>>,
}
