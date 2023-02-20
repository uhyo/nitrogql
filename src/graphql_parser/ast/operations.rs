#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}
