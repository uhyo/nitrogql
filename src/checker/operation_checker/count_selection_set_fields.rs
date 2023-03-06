use crate::graphql_parser::ast::selection_set::{Selection, SelectionSet};

use super::fragment_map::FragmentMap;

pub fn selection_set_has_more_than_one_fields(
    fragment_map: &FragmentMap,
    selection_set: &SelectionSet,
) -> bool {
    selection_set_has_more_than_one_fields_impl(fragment_map, selection_set, &[]) > 1
}

fn selection_set_has_more_than_one_fields_impl(
    fragment_map: &FragmentMap,
    selection_set: &SelectionSet,
    seen_fragments: &[&str],
) -> usize {
    let mut count = 0;
    for selection in selection_set.selections.iter() {
        match selection {
            Selection::Field(_) => {
                count += 1;
            }
            Selection::FragmentSpread(fragment_spread) => {
                if seen_fragments.contains(&fragment_spread.fragment_name.name) {
                    // prevent infinite recursions
                    continue;
                }
                let fragment_def = fragment_map.get(fragment_spread.fragment_name.name);
                match fragment_def {
                    None => {
                        // This should be handled elsewhere
                        continue;
                    }
                    Some(f) => {
                        let seen_fragments: Vec<_> = seen_fragments
                            .into_iter()
                            .map(|s| *s)
                            .chain(vec![fragment_spread.fragment_name.name])
                            .collect();
                        count += selection_set_has_more_than_one_fields_impl(
                            fragment_map,
                            &f.selection_set,
                            &seen_fragments,
                        );
                    }
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                count += selection_set_has_more_than_one_fields_impl(
                    fragment_map,
                    &inline_fragment.selection_set,
                    seen_fragments,
                );
            }
        }
    }
    count
}
