use super::{utils::PairExt, Rule};
use pest::iterators::Pair;

use nitrogql_ast::base::Variable;

pub fn build_variable(pair: Pair<Rule>) -> Variable {
    let position = pair.to_pos();
    let name = pair.only_child();
    Variable {
        name: name.as_str(),
        position,
    }
}
