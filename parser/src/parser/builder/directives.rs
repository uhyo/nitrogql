use super::{utils::PairExt, value::build_arguments, Rule};
use pest::iterators::Pair;

use crate::parts;
use nitrogql_ast::directive::Directive;

pub fn build_directives(pair: Pair<Rule>) -> Vec<Directive> {
    pair.all_children(Rule::Directive)
        .into_iter()
        .map(|pair| {
            let position = pair.to_pos();
            let (name, arguments) = parts!(
                pair,
                Name,
                Arguments opt
            );
            Directive {
                position,
                name: name.to_ident(),
                arguments: arguments.map(build_arguments),
            }
        })
        .collect()
}
