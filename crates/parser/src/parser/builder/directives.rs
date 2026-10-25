use super::{Rule, utils::PairExt, value::build_arguments};
use pest::iterators::Pair;

use crate::parts;
use nitrogql_ast::directive::Directive;

pub fn build_directives(pair: Pair<Rule>, file: usize) -> Vec<Directive> {
    pair.all_children(Rule::Directive)
        .into_iter()
        .map(|pair| {
            let position = pair.to_pos(file);
            let (name, arguments) = parts!(
                pair,
                Name,
                Arguments opt
            );
            Directive {
                position,
                name: name.to_ident(file),
                arguments: arguments.map(|pair| build_arguments(pair, file)),
            }
        })
        .collect()
}
