use super::{utils::PairExt, value::build_arguments, Rule};
use pest::iterators::Pair;

use crate::{
    graphql_parser::ast::{base::Pos, directive::Directive},
    parts,
};

pub fn build_directives(pair: Pair<Rule>) -> Vec<Directive> {
    pair.all_children(Rule::Directive)
        .into_iter()
        .map(|pair| {
            let position: Pos = (&pair).into();
            let (name, arguments) = parts!(
                pair,
                Name,
                Arguments opt
            );
            Directive {
                position,
                name: name.into(),
                arguments: arguments.map(build_arguments),
            }
        })
        .collect()
}
