use super::Rule;
use pest::iterators::Pair;

use crate::graphql_parser::ast::base::Variable;

pub fn build_variable(pair: Pair<Rule>) -> Variable {
    Variable {
        name: pair.as_str(),
        position: (&pair).into(),
    }
}
