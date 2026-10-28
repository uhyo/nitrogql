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
        // The error belongs to the definition whose span contains it.
        let (definition_pos, _, _) = definitions.iter().find(|(def_pos, _, _)| {
            def_pos.file == pos.file && def_pos.start <= pos.start && pos.start < def_pos.end
        })?;
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use nitrogql_ast::{OperationDocument, base::Pos, operation::ExecutableDefinition};
    use nitrogql_error::PositionedError;
    use nitrogql_parser::parse_operation_document;
    use nitrogql_semantics::resolve_operation_extensions;

    use super::fallback_definitions;

    const DOCUMENT: &str = "query A { ...F }
query B { b }
# between definitions
fragment F on T { f }
# after definitions
";

    fn parse(source: &str) -> OperationDocument<'_> {
        let document = parse_operation_document(source, 0).unwrap();
        resolve_operation_extensions(document).unwrap().0
    }

    /// Creates an error positioned at the first occurrence of `needle` in `source`.
    fn error_at(source: &str, needle: &str, file: usize) -> PositionedError {
        let start = source.find(needle).unwrap();
        let line = source[..start].matches('\n').count();
        let column = source[..start]
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .count();
        let pos = Pos::new(line, column, start..start + needle.len(), file);
        PositionedError::new(anyhow::anyhow!("error"), Some(pos), vec![])
    }

    /// Returns the names of definitions that fall back, in document order.
    fn fallback_names(document: &OperationDocument, fallback: &HashSet<Pos>) -> Vec<String> {
        document
            .definitions
            .iter()
            .filter_map(|def| match def {
                ExecutableDefinition::OperationDefinition(op) => fallback
                    .contains(&op.position)
                    .then(|| op.name.as_ref().unwrap().name.to_owned()),
                ExecutableDefinition::FragmentDefinition(fragment) => fallback
                    .contains(&fragment.position)
                    .then(|| fragment.name.name.to_owned()),
            })
            .collect()
    }

    #[test]
    fn error_in_definition() {
        let document = parse(DOCUMENT);
        let fallback = fallback_definitions(&document, &[error_at(DOCUMENT, "b }", 0)]).unwrap();
        assert_eq!(fallback_names(&document, &fallback), vec!["B"]);
    }

    #[test]
    fn error_in_spread_fragment() {
        let document = parse(DOCUMENT);
        let fallback = fallback_definitions(&document, &[error_at(DOCUMENT, "f }", 0)]).unwrap();
        assert_eq!(fallback_names(&document, &fallback), vec!["A", "F"]);
    }

    #[test]
    fn error_between_definitions() {
        let document = parse(DOCUMENT);
        let errors = [error_at(DOCUMENT, "# between", 0)];
        assert_eq!(fallback_definitions(&document, &errors), None);
    }

    #[test]
    fn error_after_definitions() {
        let document = parse(DOCUMENT);
        let errors = [error_at(DOCUMENT, "# after", 0)];
        assert_eq!(fallback_definitions(&document, &errors), None);
    }

    #[test]
    fn error_in_other_file() {
        let document = parse(DOCUMENT);
        let errors = [error_at(DOCUMENT, "b }", 1)];
        assert_eq!(fallback_definitions(&document, &errors), None);
    }
}
