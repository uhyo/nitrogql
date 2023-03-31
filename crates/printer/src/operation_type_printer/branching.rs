use graphql_type_system::ObjectDefinition;
use nitrogql_ast::base::Pos;

/// Branching condition used for printing types of selection sets.
pub struct BranchingCondition<'a, S> {
    pub parent_obj: &'a ObjectDefinition<S, Pos>,
}
