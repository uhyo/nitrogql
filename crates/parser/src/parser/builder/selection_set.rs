use super::{Rule, directives::build_directives, utils::PairExt, value::build_arguments};
use pest::iterators::Pair;

use crate::parts;
use nitrogql_ast::selection_set::{Field, FragmentSpread, InlineFragment, Selection, SelectionSet};

pub fn build_selection_set(pair: Pair<Rule>) -> SelectionSet {
    let position = pair.to_pos();
    SelectionSet {
        position,
        selections: pair
            .all_children(Rule::Selection)
            .into_iter()
            .map(|pair| {
                let pair = pair.only_child();
                match pair.as_rule() {
                    Rule::Field => Selection::Field(build_field(pair)),
                    Rule::FragmentSpread => Selection::FragmentSpread(build_fragment_spread(pair)),
                    Rule::InlineFragment => Selection::InlineFragment(build_inline_fragment(pair)),
                    rule => panic!("Unexpected rule {:?} as a child of Selection", rule),
                }
            })
            .collect(),
    }
}

fn build_field(pair: Pair<Rule>) -> Field {
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
            name_pair.to_ident()
        }),
        name: name.to_ident(),
        arguments: arguments.map(build_arguments),
        directives: directives.map_or(vec![], build_directives),
        selection_set: selection_set.map(build_selection_set),
    }
}

fn build_fragment_spread(pair: Pair<Rule>) -> FragmentSpread {
    let position = pair.to_pos();
    let (name, directives) = parts!(
        pair,
        FragmentName,
        Directives opt
    );
    FragmentSpread {
        position,
        fragment_name: name.to_ident(),
        directives: directives.map_or(vec![], build_directives),
    }
}

fn build_inline_fragment(pair: Pair<Rule>) -> InlineFragment {
    let position = pair.to_pos();
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
            name.to_ident()
        }),
        directives: directives.map_or(vec![], build_directives),
        selection_set: build_selection_set(selection_set),
    }
}
