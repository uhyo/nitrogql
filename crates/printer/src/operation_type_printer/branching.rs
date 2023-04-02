use graphql_type_system::ObjectDefinition;
use nitrogql_ast::base::Pos;

/// Branching condition used for printing types of selection sets.
#[derive(Debug)]
pub struct BranchingCondition<'a, S> {
    /// Concrete object type on which selection set is applied.
    pub parent_obj: &'a ObjectDefinition<S, Pos>,
    /// Values of boolean variables that are used in `if` directives.
    pub boolean_variables: Vec<(&'a str, bool)>,
}
