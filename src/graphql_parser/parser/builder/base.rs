use super::{utils::PairExt, Rule};
use pest::iterators::Pair;

use crate::graphql_parser::ast::base::Variable;

pub fn build_variable(pair: Pair<Rule>) -> Variable {
    let position = (&pair).into();
    let name = pair.only_child();
    Variable {
        name: name.as_str(),
        position,
    }
}
