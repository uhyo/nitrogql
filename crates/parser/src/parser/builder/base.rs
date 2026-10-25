use super::{Rule, utils::PairExt};
use nitrogql_ast::variable::Variable;
use pest::iterators::Pair;

pub fn build_variable(pair: Pair<Rule>, file: usize) -> Variable {
    let position = pair.to_pos(file);
    let name = pair.only_child();
    Variable {
        name: name.as_str(),
        position,
    }
}
