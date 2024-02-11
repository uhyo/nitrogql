use std::collections::HashSet;

use graphql_type_system::Text;

use super::selection_tree::{
    SelectionTree, SelectionTreeBranch, SelectionTreeEmptyLeaf, SelectionTreeField,
    SelectionTreeObject,
};

/// Merges selection tree fields of same name into one.
/// If a field with the same name already exists, they are deeply merged into a single field.
pub fn deep_merge_selection_tree<'src, S: Text<'src>>(
    fields: impl IntoIterator<Item = SelectionTreeField<S>>,
) -> Vec<SelectionTreeField<S>> {
    let mut seen_fields = HashSet::<S>::new();
    let mut new_fields = Vec::<SelectionTreeField<S>>::new();
    for field in fields {
        if seen_fields.contains(field.name()) {
            let existing = new_fields
                .iter_mut()
                .find(|f| f.name() == field.name())
                .expect("field was just inserted");
            let existing_field = std::mem::replace(
                existing,
                SelectionTreeField::Empty(SelectionTreeEmptyLeaf {
                    name: field.name().clone(),
                }),
            );
            let new_field = merge_fields(existing_field, field);
            *existing = new_field;
        } else {
            seen_fields.insert(field.name().clone());
            new_fields.push(field);
        }
    }
    new_fields
}

fn merge_fields<'src, S: Text<'src>>(
    left: SelectionTreeField<S>,
    right: SelectionTreeField<S>,
) -> SelectionTreeField<S> {
    assert_eq!(
        left.name(),
        right.name(),
        "Cannot merge fields of different names"
    );
    match (left, right) {
        (SelectionTreeField::Empty(left), SelectionTreeField::Empty(_)) => {
            SelectionTreeField::Empty(SelectionTreeEmptyLeaf { name: left.name })
        }
        (SelectionTreeField::Leaf(left), SelectionTreeField::Leaf(right)) => {
            assert_eq!(
                left.name, right.name,
                "Cannot merge fields of different names"
            );
            SelectionTreeField::Leaf(left)
        }
        (SelectionTreeField::Object(left), SelectionTreeField::Object(right)) => {
            SelectionTreeField::Object(SelectionTreeObject {
                name: left.name,
                selection: merge_selection_trees(left.selection, right.selection),
            })
        }
        _ => panic!("Cannot merge fields of different types"),
    }
}

fn merge_selection_trees<'src, S: Text<'src>>(
    left: SelectionTree<S>,
    right: SelectionTree<S>,
) -> SelectionTree<S> {
    match (left, right) {
        (SelectionTree::NonNull(left), SelectionTree::NonNull(right)) => {
            SelectionTree::NonNull(Box::new(merge_selection_trees(*left, *right)))
        }
        (SelectionTree::List(left), SelectionTree::List(right)) => {
            SelectionTree::List(Box::new(merge_selection_trees(*left, *right)))
        }
        (SelectionTree::Object(left), SelectionTree::Object(right)) => {
            let mut new_selection = Vec::new();
            for left_branch in left {
                let right_branch = right
                    .iter()
                    .find(|b| b.type_name == left_branch.type_name)
                    .cloned();
                match right_branch {
                    Some(right_branch) => {
                        new_selection.push(SelectionTreeBranch {
                            type_name: left_branch.type_name,
                            unaliased_fields: deep_merge_selection_tree(
                                left_branch
                                    .unaliased_fields
                                    .into_iter()
                                    .chain(right_branch.unaliased_fields),
                            ),
                            aliased_fields: deep_merge_selection_tree(
                                left_branch
                                    .aliased_fields
                                    .into_iter()
                                    .chain(right_branch.aliased_fields),
                            ),
                        });
                    }
                    None => {
                        new_selection.push(left_branch);
                    }
                }
            }
            for right_branch in right {
                if !new_selection
                    .iter()
                    .any(|b| b.type_name == right_branch.type_name)
                {
                    new_selection.push(right_branch);
                }
            }
            SelectionTree::Object(new_selection)
        }
        _ => panic!("Cannot merge selection trees of different types"),
    }
}
