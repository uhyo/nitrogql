use graphql_type_system::{ObjectDefinition, Schema, Text};
use nitrogql_ast::{
    operation::FragmentDefinition,
    selection_set::{Selection, SelectionSet},
};

/// Returns an iterator over possible object types that implements given interface.
pub fn interface_implementers<'a, 'src, S: Text<'src>, OriginalNode>(
    schema: &'a Schema<S, OriginalNode>,
    interface_name: &'a str,
) -> impl Iterator<Item = &'a ObjectDefinition<S, OriginalNode>> + 'a {
    schema.iter_types().filter_map(move |(_, def)| {
        def.as_object().filter(|obj_def| {
            obj_def
                .interfaces
                .iter()
                .any(|imp| imp.inner_ref().borrow() == interface_name)
        })
    })
}

/// Returns the names of all Fragments involved in given selection set.
pub fn fragment_names_in_selection_set<'a, 'src>(
    selection_set: &'a SelectionSet<'src>,
    get_fragment: impl Fn(&'a str) -> Option<&'a FragmentDefinition<'src>>,
) -> Vec<&'a str> {
    let mut names = Vec::new();
    rec(selection_set, &get_fragment, &mut names);
    return names;

    fn rec<'a, 'src>(
        selection_set: &'a SelectionSet<'src>,
        get_fragment: &impl Fn(&'a str) -> Option<&'a FragmentDefinition<'src>>,
        names: &mut Vec<&'a str>,
    ) {
        for selection in selection_set.selections.iter() {
            match selection {
                Selection::Field(field) => {
                    if let Some(selection_set) = field.selection_set.as_ref() {
                        rec(selection_set, get_fragment, names);
                    }
                }
                Selection::FragmentSpread(fragment_spread) => {
                    if names.contains(&fragment_spread.fragment_name.name) {
                        continue;
                    }
                    names.push(fragment_spread.fragment_name.name);
                    let Some(fragment) = get_fragment(fragment_spread.fragment_name.name) else {
                        continue;
                    };
                    rec(&fragment.selection_set, get_fragment, names);
                }
                Selection::InlineFragment(inline_fragment) => {
                    rec(&inline_fragment.selection_set, get_fragment, names);
                }
            }
        }
    }
}
