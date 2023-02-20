use self::operations::OperationType;

pub mod operations;
#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    pub definitions: Vec<OperationDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct OperationDefinition<'a> {
    pub operation_type: OperationType,
    pub source: &'a str,
}
