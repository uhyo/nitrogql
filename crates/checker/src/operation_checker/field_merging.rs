//! Implements the Field Selection Merging rule.
//! https://spec.graphql.org/October2021/#sec-Field-Selection-Merging

use std::{borrow::Cow, collections::HashMap};

use graphql_type_system::{Field, Node, Text, Type, TypeDefinition};
use nitrogql_ast::{
    base::Pos,
    selection_set::{Field as SelectionField, Selection, SelectionSet},
    value::{Arguments, Value},
};
use nitrogql_semantics::direct_fields_of_output_type;

use super::{OperationCheckContext, fragment_map::FragmentMap};
use crate::error::{CheckError, CheckErrorMessage, FieldConflictReason};

/// Checks that fields with the same response name can be merged.
/// Results of comparing two fields are cached, so one checker should be used for a whole document.
pub struct FieldMergeChecker<'a, 'src, S> {
    fragment_map: &'a FragmentMap<'a, 'src>,
    context: &'a OperationCheckContext<'a, 'src, S>,
    /// Cache of compared field pairs.
    /// Key is (first field, second field, whether their parents are mutually exclusive).
    cache: HashMap<FieldPair<'src>, Option<Conflict>>,
}

type FieldPair<'src> = (
    *const SelectionField<'src>,
    *const SelectionField<'src>,
    bool,
);

/// A field collected from a selection set, together with its context.
struct CollectedField<'a, 'src, S: Clone> {
    /// Type in which this field is selected.
    parent_type: &'a Node<TypeDefinition<S, Pos>, Pos>,
    selection: &'a SelectionField<'src>,
    /// Definition of the field. None if the field is not found (reported elsewhere).
    definition: Option<Cow<'a, Field<S, Pos>>>,
    /// Position of the fragment spread in the checked selection set
    /// through which this field was collected.
    spread: Option<Pos>,
}

/// Fields collected from a selection set, grouped by response name in order of appearance.
struct CollectedFields<'a, 'src, S: Clone> {
    groups: Vec<(&'a str, Vec<CollectedField<'a, 'src, S>>)>,
    indices: HashMap<&'a str, usize>,
}

impl<'a, 'src, S: Clone> CollectedFields<'a, 'src, S> {
    fn new() -> Self {
        Self {
            groups: vec![],
            indices: HashMap::new(),
        }
    }

    fn push(&mut self, response_name: &'a str, field: CollectedField<'a, 'src, S>) {
        match self.indices.get(response_name) {
            Some(&idx) => self.groups[idx].1.push(field),
            None => {
                self.indices.insert(response_name, self.groups.len());
                self.groups.push((response_name, vec![field]));
            }
        }
    }

    fn get(&self, response_name: &str) -> Option<&[CollectedField<'a, 'src, S>]> {
        self.indices
            .get(response_name)
            .map(|&idx| &self.groups[idx].1[..])
    }
}

#[derive(Clone, Debug)]
struct Conflict {
    reason: FieldConflictReason,
    /// Response names and positions of the innermost conflicting fields.
    fields: [(String, Pos); 2],
}

impl<'a, 'src, S: Text<'src>> FieldMergeChecker<'a, 'src, S> {
    pub fn new(
        fragment_map: &'a FragmentMap<'a, 'src>,
        context: &'a OperationCheckContext<'a, 'src, S>,
    ) -> Self {
        Self {
            fragment_map,
            context,
            cache: HashMap::new(),
        }
    }

    /// Checks given selection set and all selection sets nested in it.
    /// Fragments spread from it are not checked on their own; they should be checked separately.
    pub fn check_selection_set(
        &mut self,
        parent_type: &'a Node<TypeDefinition<S, Pos>, Pos>,
        selection_set: &'a SelectionSet<'src>,
        result: &mut Vec<CheckError>,
    ) {
        let mut fields = CollectedFields::new();
        self.collect_fields(parent_type, selection_set, None, &mut vec![], &mut fields);
        self.find_conflicts_within(&fields, result);
        self.check_nested_selection_sets(parent_type, selection_set, result);
    }

    fn check_nested_selection_sets(
        &mut self,
        parent_type: &'a Node<TypeDefinition<S, Pos>, Pos>,
        selection_set: &'a SelectionSet<'src>,
        result: &mut Vec<CheckError>,
    ) {
        for selection in selection_set.selections.iter() {
            match selection {
                Selection::Field(field) => {
                    let Some(ref selection_set) = field.selection_set else {
                        continue;
                    };
                    let Some(field_type) = find_field_definition(parent_type, field)
                        .and_then(|def| self.get_type(def.r#type.unwrapped()))
                    else {
                        continue;
                    };
                    self.check_selection_set(field_type, selection_set, result);
                }
                Selection::InlineFragment(fragment) => {
                    // Fields in inline fragments are already collected as part of this selection set.
                    let Some(fragment_type) = self.inline_fragment_type(
                        parent_type,
                        fragment.type_condition.map(|cond| cond.name),
                    ) else {
                        continue;
                    };
                    self.check_nested_selection_sets(
                        fragment_type,
                        &fragment.selection_set,
                        result,
                    );
                }
                Selection::FragmentSpread(_) => {}
            }
        }
    }

    fn collect_fields(
        &self,
        parent_type: &'a Node<TypeDefinition<S, Pos>, Pos>,
        selection_set: &'a SelectionSet<'src>,
        spread: Option<Pos>,
        visited_fragments: &mut Vec<&'a str>,
        result: &mut CollectedFields<'a, 'src, S>,
    ) {
        for selection in selection_set.selections.iter() {
            match selection {
                Selection::Field(field) => {
                    let response_name = field.alias.unwrap_or(field.name).name;
                    result.push(
                        response_name,
                        CollectedField {
                            parent_type,
                            selection: field,
                            definition: find_field_definition(parent_type, field),
                            spread,
                        },
                    );
                }
                Selection::InlineFragment(fragment) => {
                    let Some(fragment_type) = self.inline_fragment_type(
                        parent_type,
                        fragment.type_condition.map(|cond| cond.name),
                    ) else {
                        continue;
                    };
                    self.collect_fields(
                        fragment_type,
                        &fragment.selection_set,
                        spread,
                        visited_fragments,
                        result,
                    );
                }
                Selection::FragmentSpread(fragment_spread) => {
                    let name = fragment_spread.fragment_name.name;
                    if visited_fragments.contains(&name) {
                        continue;
                    }
                    visited_fragments.push(name);
                    // Unknown fragments are reported elsewhere.
                    // Imported fragments whose definitions are not available cannot be checked here.
                    let Some(fragment) = self.fragment_map.get(name) else {
                        continue;
                    };
                    let Some(fragment_type) = self.get_type(fragment.type_condition.name) else {
                        continue;
                    };
                    self.collect_fields(
                        fragment_type,
                        &fragment.selection_set,
                        spread.or(Some(fragment_spread.position)),
                        visited_fragments,
                        result,
                    );
                }
            }
        }
    }

    fn find_conflicts_within(
        &mut self,
        fields: &CollectedFields<'a, 'src, S>,
        result: &mut Vec<CheckError>,
    ) {
        for (response_name, group) in fields.groups.iter() {
            for (idx, second) in group.iter().enumerate() {
                for first in group[..idx].iter() {
                    if first.spread.is_some() && first.spread == second.spread {
                        // Both fields come from the same fragment spread.
                        // This is checked when the fragment itself is checked.
                        continue;
                    }
                    let Some(conflict) = self.find_conflict(first, second, false) else {
                        continue;
                    };
                    // Report the error in the checked selection set:
                    // at a field selected directly in it if possible, otherwise at the fragment spread.
                    let position = match (first.spread, second.spread) {
                        (_, None) => field_position(second.selection),
                        (None, Some(_)) => field_position(first.selection),
                        (Some(_), Some(spread)) => spread,
                    };
                    let additional_info = conflict
                        .fields
                        .into_iter()
                        .filter(|(_, pos)| *pos != position)
                        .map(|(name, pos)| (pos, CheckErrorMessage::ConflictingField { name }));
                    result.push(
                        CheckErrorMessage::FieldsConflict {
                            name: (*response_name).to_owned(),
                            reason: conflict.reason,
                        }
                        .with_pos(position)
                        .with_additional_info(additional_info),
                    );
                }
            }
        }
    }

    /// Checks whether two fields with the same response name can be merged.
    /// `parents_are_exclusive` means that the two fields are never selected for the same object.
    fn find_conflict(
        &mut self,
        first: &CollectedField<'a, 'src, S>,
        second: &CollectedField<'a, 'src, S>,
        parents_are_exclusive: bool,
    ) -> Option<Conflict> {
        if std::ptr::eq(first.selection, second.selection) {
            // A field never conflicts with itself.
            // Conflicts in its selection set are reported where the selection set is checked.
            return None;
        }
        let key = (
            first.selection as *const _,
            second.selection as *const _,
            parents_are_exclusive,
        );
        if let Some(conflict) = self.cache.get(&key) {
            return conflict.clone();
        }
        let conflict = self.find_conflict_uncached(first, second, parents_are_exclusive);
        self.cache.insert(key, conflict.clone());
        conflict
    }

    fn find_conflict_uncached(
        &mut self,
        first: &CollectedField<'a, 'src, S>,
        second: &CollectedField<'a, 'src, S>,
        parents_are_exclusive: bool,
    ) -> Option<Conflict> {
        let conflict = |reason| Conflict {
            reason,
            fields: [first, second].map(|field| {
                (
                    field
                        .selection
                        .alias
                        .unwrap_or(field.selection.name)
                        .name
                        .to_owned(),
                    field_position(field.selection),
                )
            }),
        };
        // Fields in different object types are never selected for the same object.
        // In this case only their response shapes need to be the same.
        let parents_are_exclusive = parents_are_exclusive
            || (first.parent_type.name() != second.parent_type.name()
                && first.parent_type.as_object().is_some()
                && second.parent_type.as_object().is_some());
        if !parents_are_exclusive {
            if first.selection.name.name != second.selection.name.name {
                return Some(conflict(FieldConflictReason::DifferentFields {
                    first: first.selection.name.name.to_owned(),
                    second: second.selection.name.name.to_owned(),
                }));
            }
            if !same_arguments(
                first.selection.arguments.as_ref(),
                second.selection.arguments.as_ref(),
            ) {
                return Some(conflict(FieldConflictReason::DifferentArguments));
            }
        }

        let (Some(first_definition), Some(second_definition)) =
            (&first.definition, &second.definition)
        else {
            return None;
        };
        if self.types_conflict(&first_definition.r#type, &second_definition.r#type) {
            return Some(conflict(FieldConflictReason::DifferentTypes {
                first: first_definition.r#type.to_string(),
                second: second_definition.r#type.to_string(),
            }));
        }

        let (Some(first_selection_set), Some(second_selection_set)) = (
            &first.selection.selection_set,
            &second.selection.selection_set,
        ) else {
            return None;
        };
        let (Some(first_type), Some(second_type)) = (
            self.get_type(first_definition.r#type.unwrapped()),
            self.get_type(second_definition.r#type.unwrapped()),
        ) else {
            return None;
        };
        let mut first_subfields = CollectedFields::new();
        self.collect_fields(
            first_type,
            first_selection_set,
            None,
            &mut vec![],
            &mut first_subfields,
        );
        let mut second_subfields = CollectedFields::new();
        self.collect_fields(
            second_type,
            second_selection_set,
            None,
            &mut vec![],
            &mut second_subfields,
        );
        // Conflicts within each selection set are reported where the selection set is checked.
        // Here we only look for conflicts between the two.
        for (response_name, first_group) in first_subfields.groups.iter() {
            let Some(second_group) = second_subfields.get(response_name) else {
                continue;
            };
            for first_subfield in first_group.iter() {
                for second_subfield in second_group.iter() {
                    if let Some(sub_conflict) =
                        self.find_conflict(first_subfield, second_subfield, parents_are_exclusive)
                    {
                        return Some(Conflict {
                            reason: FieldConflictReason::Subfields {
                                name: (*response_name).to_owned(),
                                reason: Box::new(sub_conflict.reason),
                            },
                            fields: sub_conflict.fields,
                        });
                    }
                }
            }
        }
        None
    }

    /// Checks whether two field types have different response shapes.
    fn types_conflict(&self, first: &Type<S, Pos>, second: &Type<S, Pos>) -> bool {
        match (first, second) {
            (Type::NonNull(first), Type::NonNull(second)) => {
                self.types_conflict(first.as_inner(), second.as_inner())
            }
            (Type::NonNull(_), _) | (_, Type::NonNull(_)) => true,
            (Type::List(first), Type::List(second)) => {
                self.types_conflict(first.as_inner(), second.as_inner())
            }
            (Type::List(_), _) | (_, Type::List(_)) => true,
            (Type::Named(first), Type::Named(second)) => {
                let is_leaf = |name: &str| {
                    self.get_type(name).is_some_and(|ty| {
                        matches!(**ty, TypeDefinition::Scalar(_) | TypeDefinition::Enum(_))
                    })
                };
                // Composite types are compared by their subfields.
                (is_leaf(first) || is_leaf(second)) && ***first != ***second
            }
        }
    }

    fn inline_fragment_type(
        &self,
        parent_type: &'a Node<TypeDefinition<S, Pos>, Pos>,
        type_condition: Option<&str>,
    ) -> Option<&'a Node<TypeDefinition<S, Pos>, Pos>> {
        match type_condition {
            None => Some(parent_type),
            // Unknown types are reported elsewhere.
            Some(name) => self.get_type(name),
        }
    }

    fn get_type(&self, name: &str) -> Option<&'a Node<TypeDefinition<S, Pos>, Pos>> {
        self.context.definitions.get_type(name)
    }
}

fn find_field_definition<'a, 'src, S: Text<'src>>(
    parent_type: &'a Node<TypeDefinition<S, Pos>, Pos>,
    field: &SelectionField<'src>,
) -> Option<Cow<'a, Field<S, Pos>>> {
    direct_fields_of_output_type(&**parent_type)?
        .into_iter()
        .find(|def| def.name == field.name.name)
}

/// Position of a field selection, which is the position of its response name.
fn field_position(field: &SelectionField) -> Pos {
    field.alias.unwrap_or(field.name).position
}

fn same_arguments(first: Option<&Arguments>, second: Option<&Arguments>) -> bool {
    let first = first.map_or(&[][..], |args| &args.arguments[..]);
    let second = second.map_or(&[][..], |args| &args.arguments[..]);
    first.len() == second.len()
        && first.iter().all(|(name, value)| {
            second.iter().any(|(other_name, other_value)| {
                name.name == other_name.name && same_value(value, other_value)
            })
        })
}

fn same_value(first: &Value, second: &Value) -> bool {
    match (first, second) {
        (Value::Variable(first), Value::Variable(second)) => first.name == second.name,
        (Value::IntValue(first), Value::IntValue(second)) => first.value == second.value,
        (Value::FloatValue(first), Value::FloatValue(second)) => first.value == second.value,
        (Value::StringValue(first), Value::StringValue(second)) => first.value == second.value,
        (Value::BooleanValue(first), Value::BooleanValue(second)) => first.value == second.value,
        (Value::NullValue(_), Value::NullValue(_)) => true,
        (Value::EnumValue(first), Value::EnumValue(second)) => first.value == second.value,
        (Value::ListValue(first), Value::ListValue(second)) => {
            first.values.len() == second.values.len()
                && first
                    .values
                    .iter()
                    .zip(second.values.iter())
                    .all(|(first, second)| same_value(first, second))
        }
        (Value::ObjectValue(first), Value::ObjectValue(second)) => {
            first.fields.len() == second.fields.len()
                && first.fields.iter().all(|(name, value)| {
                    second.fields.iter().any(|(other_name, other_value)| {
                        name.name == other_name.name && same_value(value, other_value)
                    })
                })
        }
        _ => false,
    }
}
