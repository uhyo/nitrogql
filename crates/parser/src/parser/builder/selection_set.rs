use super::{Rule, directives::build_directives, utils::PairExt, value::build_arguments};
use pest::iterators::Pair;

use crate::parts;
use nitrogql_ast::selection_set::{Field, FragmentSpread, InlineFragment, Selection, SelectionSet};

pub fn build_selection_set(pair: Pair<Rule>, file: usize) -> SelectionSet {
    let position = pair.to_pos(file);
    SelectionSet {
        position,
        selections: pair
            .all_children(Rule::Selection)
            .into_iter()
            .map(|pair| {
                let pair = pair.only_child();
                match pair.as_rule() {
                    Rule::Field => Selection::Field(build_field(pair, file)),
                    Rule::FragmentSpread => {
                        Selection::FragmentSpread(build_fragment_spread(pair, file))
                    }
                    Rule::InlineFragment => {
                        Selection::InlineFragment(build_inline_fragment(pair, file))
                    }
                    rule => panic!("Unexpected rule {:?} as a child of Selection", rule),
                }
            })
            .collect(),
    }
}

fn build_field(pair: Pair<Rule>, file: usize) -> Field {
    let (alias, name, arguments, directives, selection_set) = parts!(
        pair,
        Alias opt,
        Name,
        Arguments opt,
        Directives opt,
        SelectionSet opt
    );
    Field {
        alias: alias.map(|pair| {
            let name_pair = pair.only_child();
            name_pair.to_ident(file)
        }),
        name: name.to_ident(file),
        arguments: arguments.map(|pair| build_arguments(pair, file)),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        selection_set: selection_set.map(|pair| build_selection_set(pair, file)),
    }
}

fn build_fragment_spread(pair: Pair<Rule>, file: usize) -> FragmentSpread {
    let position = pair.to_pos(file);
    let (name, directives) = parts!(
        pair,
        FragmentName,
        Directives opt
    );
    FragmentSpread {
        position,
        fragment_name: name.to_ident(file),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
    }
}

fn build_inline_fragment(pair: Pair<Rule>, file: usize) -> InlineFragment {
    let position = pair.to_pos(file);
    let (type_condition, directives, selection_set) = parts!(
        pair,
        TypeCondition opt,
        Directives opt,
        SelectionSet
    );
    InlineFragment {
        position,
        type_condition: type_condition.map(|type_condition_pair| {
            let (_, name) = parts!(type_condition_pair, KEYWORD_on, NamedType);
            name.to_ident(file)
        }),
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        selection_set: build_selection_set(selection_set, file),
    }
}
