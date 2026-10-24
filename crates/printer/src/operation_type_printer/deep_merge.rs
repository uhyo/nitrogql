use std::collections::HashSet;

use graphql_type_system::Text;

use super::selection_tree::{
    SelectionTree, SelectionTreeBranch, SelectionTreeEmptyLeaf, SelectionTreeField,
    SelectionTreeObject,
};

/// Error from merging fields whose selections have different shapes.
#[derive(Debug)]
pub struct FieldShapeMismatch<S> {
    /// Response key of the merged fields at the level where merging failed.
    pub key: S,
    /// Name of the field (possibly nested in `key`) whose shapes differ.
    pub name: S,
}

/// Merges selection tree fields of same name into one.
/// If a field with the same name already exists, they are deeply merged into a single field.
pub fn deep_merge_selection_tree<'src, S: Text<'src>>(
    fields: impl IntoIterator<Item = SelectionTreeField<S>>,
) -> Result<Vec<SelectionTreeField<S>>, FieldShapeMismatch<S>> {
    let mut seen_fields = HashSet::<S>::new();
    let mut new_fields = Vec::<SelectionTreeField<S>>::new();
    for field in fields {
        if seen_fields.contains(field.name()) {
            let existing = new_fields
                .iter_mut()
                .find(|f| f.name() == field.name())
                .expect("field was just inserted");
            let key = field.name().clone();
            let existing_field = std::mem::replace(
                existing,
                SelectionTreeField::Empty(SelectionTreeEmptyLeaf { name: key.clone() }),
            );
            let new_field = merge_fields(existing_field, field)
                .map_err(|name| FieldShapeMismatch { key, name })?;
            *existing = new_field;
        } else {
            seen_fields.insert(field.name().clone());
            new_fields.push(field);
        }
    }
    Ok(new_fields)
}

/// Merges two fields of the same name.
/// On error, returns the name of the (possibly nested) field whose shapes differ.
fn merge_fields<'src, S: Text<'src>>(
    left: SelectionTreeField<S>,
    right: SelectionTreeField<S>,
) -> Result<SelectionTreeField<S>, S> {
    assert_eq!(
        left.name(),
        right.name(),
        "Cannot merge fields of different names"
    );
    Ok(match (left, right) {
        (SelectionTreeField::Empty(left), SelectionTreeField::Empty(_)) => {
            SelectionTreeField::Empty(SelectionTreeEmptyLeaf { name: left.name })
        }
        (SelectionTreeField::Leaf(left), SelectionTreeField::Leaf(_)) => {
            SelectionTreeField::Leaf(left)
        }
        (SelectionTreeField::Leaf(left), SelectionTreeField::Empty(_)) => {
            SelectionTreeField::Leaf(left)
        }
        (SelectionTreeField::Empty(_), SelectionTreeField::Leaf(right)) => {
            SelectionTreeField::Leaf(right)
        }
        (SelectionTreeField::Object(left), SelectionTreeField::Object(right)) => {
            let Some(selection) = merge_selection_trees(left.selection, right.selection)? else {
                return Err(left.name);
            };
            SelectionTreeField::Object(SelectionTreeObject {
                name: left.name,
                selection,
            })
        }
        (SelectionTreeField::Empty(_), SelectionTreeField::Object(right)) => {
            SelectionTreeField::Object(right)
        }
        (SelectionTreeField::Object(left), SelectionTreeField::Empty(_)) => {
            SelectionTreeField::Object(left)
        }
        (left, _) => return Err(left.name().clone()),
    })
}

/// Merges two selection trees.
/// Returns `None` if the trees themselves have different shapes,
/// and an error with the name of the nested field whose shapes differ.
fn merge_selection_trees<'src, S: Text<'src>>(
    left: SelectionTree<S>,
    right: SelectionTree<S>,
) -> Result<Option<SelectionTree<S>>, S> {
    Ok(Some(match (left, right) {
        (SelectionTree::NonNull(left), SelectionTree::NonNull(right)) => {
            let Some(inner) = merge_selection_trees(*left, *right)? else {
                return Ok(None);
            };
            SelectionTree::NonNull(Box::new(inner))
        }
        (SelectionTree::List(left), SelectionTree::List(right)) => {
            let Some(inner) = merge_selection_trees(*left, *right)? else {
                return Ok(None);
            };
            SelectionTree::List(Box::new(inner))
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
                            )
                            .map_err(|err| err.name)?,
                            aliased_fields: deep_merge_selection_tree(
                                left_branch
                                    .aliased_fields
                                    .into_iter()
                                    .chain(right_branch.aliased_fields),
                            )
                            .map_err(|err| err.name)?,
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
        _ => return Ok(None),
    }))
}
