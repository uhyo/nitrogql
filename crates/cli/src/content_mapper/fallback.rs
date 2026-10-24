//! Decides which definitions of an operation document fall back to `any` types.

use std::collections::{HashMap, HashSet};

use nitrogql_ast::{
    OperationDocument,
    base::Pos,
    operation::ExecutableDefinition,
    selection_set::{Selection, SelectionSet},
};
use nitrogql_error::PositionedError;

/// Returns the positions of definitions that should fall back to `any` types
/// because of given errors.
///
/// A definition falls back if an error is located in it,
/// or if it (transitively) spreads a fragment that falls back.
/// Returns `None` if some error cannot be attributed to a definition;
/// the whole document should fall back then.
pub fn fallback_definitions(
    document: &OperationDocument,
    errors: &[PositionedError],
) -> Option<HashSet<Pos>> {
    let definitions = document
        .definitions
        .iter()
        .map(|def| match def {
            ExecutableDefinition::OperationDefinition(op) => (op.position, None, &op.selection_set),
            ExecutableDefinition::FragmentDefinition(fragment) => (
                fragment.position,
                Some(fragment.name.name),
                &fragment.selection_set,
            ),
        })
        .collect::<Vec<_>>();

    let mut result = HashSet::new();
    for err in errors {
        let pos = err.position().filter(|pos| !pos.builtin)?;
        // Definitions do not overlap, so the error belongs to the last definition starting before it.
        let (definition_pos, _, _) = definitions
            .iter()
            .filter(|(def_pos, _, _)| {
                def_pos.file == pos.file && (def_pos.line, def_pos.column) <= (pos.line, pos.column)
            })
            .max_by_key(|(def_pos, _, _)| (def_pos.line, def_pos.column))?;
        result.insert(*definition_pos);
    }

    let fragment_positions = definitions
        .iter()
        .filter_map(|(pos, name, _)| name.map(|name| (name, *pos)))
        .collect::<HashMap<_, _>>();
    let spreads = definitions
        .iter()
        .map(|(pos, _, selection_set)| {
            let mut spreads = vec![];
            collect_fragment_spreads(selection_set, &mut spreads);
            (*pos, spreads)
        })
        .collect::<Vec<_>>();
    loop {
        let mut changed = false;
        for (pos, spreads) in &spreads {
            if result.contains(pos) {
                continue;
            }
            let depends_on_fallback = spreads.iter().any(|name| {
                fragment_positions
                    .get(name)
                    .is_some_and(|fragment_pos| result.contains(fragment_pos))
            });
            if depends_on_fallback {
                result.insert(*pos);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    Some(result)
}

/// Collects names of fragments spread in given selection set, including nested ones.
fn collect_fragment_spreads<'src>(selection_set: &SelectionSet<'src>, result: &mut Vec<&'src str>) {
    for selection in &selection_set.selections {
        match selection {
            Selection::Field(field) => {
                if let Some(ref selection_set) = field.selection_set {
                    collect_fragment_spreads(selection_set, result);
                }
            }
            Selection::FragmentSpread(spread) => {
                result.push(spread.fragment_name.name);
            }
            Selection::InlineFragment(fragment) => {
                collect_fragment_spreads(&fragment.selection_set, result);
            }
        }
    }
}
