use super::{Rule, utils::PairExt};
use nitrogql_ast::variable::Variable;
use pest::iterators::Pair;

pub fn build_variable(pair: Pair<Rule>) -> Variable {
    let position = pair.to_pos();
    let name = pair.only_child();
    Variable {
        name: name.as_str(),
        position,
    }
}
