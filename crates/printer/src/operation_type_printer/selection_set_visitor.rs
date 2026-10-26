use graphql_type_system::Text;
use nitrogql_ast::selection_set::{Selection, SelectionSet};

use super::{
    error::{OperationTypePrinterErrorMessage, OperationTypePrinterResult},
    type_printer::QueryTypePrinterContext,
};

/// Visits all fields in selection set. Nested fragments are visited as well,
/// except for fragments defined in other modules.
pub fn visit_fields_in_selection_set<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    mut visitor: impl FnMut(&Selection<'src>),
) -> OperationTypePrinterResult<()> {
    visit_fields_in_selection_set_impl(context, selection_set, &mut vec![], &mut visitor)
}

fn visit_fields_in_selection_set_impl<'src, S: Text<'src>>(
    context: &QueryTypePrinterContext<'_, 'src, S>,
    selection_set: &SelectionSet<'src>,
    seen_fragments: &mut Vec<&'src str>,
    visitor: &mut impl FnMut(&Selection<'src>),
) -> OperationTypePrinterResult<()> {
    for sel in &selection_set.selections {
        visitor(sel);
        match sel {
            Selection::Field(_) => {}
            Selection::FragmentSpread(fragment) => {
                if seen_fragments.contains(&fragment.fragment_name.name) {
                    continue;
                }
                seen_fragments.push(fragment.fragment_name.name);
                if context.is_external_fragment(fragment.fragment_name.name) {
                    // Selections of fragments in other modules are not known here.
                    continue;
                }
                let fragment_def = context
                    .fragment_definitions
                    .get(fragment.fragment_name.name)
                    .ok_or_else(|| {
                        OperationTypePrinterErrorMessage::UnknownFragment {
                            name: fragment.fragment_name.name.to_owned(),
                        }
                        .with_pos(fragment.fragment_name.position)
                    })?;
                visit_fields_in_selection_set_impl(
                    context,
                    &fragment_def.selection_set,
                    seen_fragments,
                    visitor,
                )?;
            }
            Selection::InlineFragment(fragment) => {
                visit_fields_in_selection_set_impl(
                    context,
                    &fragment.selection_set,
                    seen_fragments,
                    visitor,
                )?;
            }
        }
    }
    Ok(())
}
