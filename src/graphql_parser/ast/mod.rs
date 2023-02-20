#[derive(Clone, Debug)]
pub struct OperationDocument<'a> {
    pub definitions: Vec<OperationDefinition<'a>>,
}

#[derive(Clone, Debug)]
pub struct OperationDefinition<'a> {
    pub source: &'a str,
}
