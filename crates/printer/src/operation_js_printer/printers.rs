use std::collections::HashMap;

use crate::{
    json_printer::{ExecutableDefinitionRef, print_to_json_string},
    utils::fragment_names_in_selection_set,
};
use nitrogql_ast::operation::{FragmentDefinition, OperationDefinition};
use sourcemap_writer::SourceMapWriter;

/// Print the runtime value of given operation.
pub fn print_operation_runtime(
    writer: &mut impl SourceMapWriter,
    operation: &OperationDefinition,
    fragments: &HashMap<&str, &FragmentDefinition>,
) {
    let fragments_to_include = fragment_names_in_selection_set(&operation.selection_set, |name| {
        fragments.get(name).copied()
    })
    .into_iter()
    .map(|name| {
        ExecutableDefinitionRef::FragmentDefinition(
            fragments.get(name).expect("fragment not found"),
        )
    });
    let this_document = vec![ExecutableDefinitionRef::OperationDefinition(operation)]
        .into_iter()
        .chain(fragments_to_include)
        .collect::<Vec<_>>();
    writer.write(&print_to_json_string(&this_document[..]));
}

/// Print the runtime value of given fragment.
pub fn print_fragment_runtime(
    writer: &mut impl SourceMapWriter,
    fragment: &FragmentDefinition,
    fragments: &HashMap<&str, &FragmentDefinition>,
) {
    let fragments_to_include = fragment_names_in_selection_set(&fragment.selection_set, |name| {
        fragments.get(name).copied()
    })
    .into_iter()
    .filter(|f| {
        // Filter out the fragment we are currently processing
        *f != fragment.name.name
    })
    .map(|name| {
        ExecutableDefinitionRef::FragmentDefinition(
            fragments.get(name).expect("fragment not found"),
        )
    });
    // Generated document is the collection of all relevant fragments,
    // and the fragment we are currently processing
    // comes first in the list
    let this_document = vec![ExecutableDefinitionRef::FragmentDefinition(fragment)]
        .into_iter()
        .chain(fragments_to_include)
        .collect::<Vec<_>>();
    writer.write(&print_to_json_string(&this_document[..]));
}
