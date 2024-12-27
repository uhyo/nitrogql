use graphql_type_system::Schema;
use nitrogql_ast::base::Pos;

pub struct OperationCheckContext<'schema, 'src, S> {
    pub definitions: &'schema Schema<S, Pos>,
    // For some reasons 'src is considered unused
    phantom: std::marker::PhantomData<&'src ()>,
}

impl<'schema, S> OperationCheckContext<'schema, '_, S> {
    pub fn new(definitions: &'schema Schema<S, Pos>) -> Self {
        Self {
            definitions,
            phantom: std::marker::PhantomData,
        }
    }
}
