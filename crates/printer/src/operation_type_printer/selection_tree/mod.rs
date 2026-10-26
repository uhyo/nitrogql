//! Selection Tree is a tree structure that represents the selection in a GraphQL query.
//! Conversion from a selection set to a tree is basically resolution of the fragments.

use graphql_type_system::Type;
use nitrogql_ast::base::Pos;

mod to_ts;

pub use to_ts::{
    GenerateSelectionTreeTypeContext, generate_selection_tree_type,
    generate_unrendered_branches_type,
};

#[derive(Debug, Clone)]
pub enum SelectionTree<S> {
    NonNull(Box<SelectionTree<S>>),
    List(Box<SelectionTree<S>>),
    Object(Vec<SelectionTreeBranch<S>>),
}

#[derive(Debug, Clone)]
pub struct SelectionTreeBranch<S> {
    /// Name of GraphQL type that this selection corresponds to.
    pub type_name: String,
    /// Values of boolean variables under which this branch is selected.
    pub variables: Vec<(String, bool)>,
    /// List of unaliased fields that are selected.
    pub unaliased_fields: Vec<SelectionTreeField<S>>,
    /// List of aliased fields that are selected.
    pub aliased_fields: Vec<SelectionTreeField<S>>,
    /// Fragments defined in other modules whose selections are merged into this branch.
    pub fragment_refs: Vec<FragmentRef<S>>,
}

impl<S> SelectionTree<S> {
    /// Returns whether this tree refers to fragments defined in other modules.
    pub fn has_fragment_refs(&self) -> bool {
        match self {
            SelectionTree::NonNull(inner) | SelectionTree::List(inner) => inner.has_fragment_refs(),
            SelectionTree::Object(branches) => branches.iter().any(|branch| {
                !branch.fragment_refs.is_empty()
                    || branch
                        .unaliased_fields
                        .iter()
                        .chain(&branch.aliased_fields)
                        .any(|field| match field {
                            SelectionTreeField::Object(object) => {
                                object.selection.has_fragment_refs()
                            }
                            _ => false,
                        })
            }),
        }
    }
}

/// Spread of a fragment defined in another module.
/// Its selection is not known here, and is merged at the type level.
#[derive(Debug, Clone)]
pub struct FragmentRef<S> {
    /// Name of the fragment.
    pub name: S,
    /// Whether the spread is skipped under the branching condition.
    pub skipped: bool,
}

#[derive(Debug, Clone)]
pub enum SelectionTreeField<S> {
    Empty(SelectionTreeEmptyLeaf<S>),
    Leaf(SelectionTreeLeaf<S>),
    Object(SelectionTreeObject<S>),
}

impl<S> SelectionTreeField<S> {
    pub fn name(&self) -> &S {
        match self {
            SelectionTreeField::Empty(empty) => &empty.name,
            SelectionTreeField::Leaf(leaf) => &leaf.name,
            SelectionTreeField::Object(object) => &object.name,
        }
    }
}

/// Empty leaf (field that is omitted from the selection under this condition).a
#[derive(Debug, Clone)]
pub struct SelectionTreeEmptyLeaf<S> {
    /// Name of the field.
    pub name: S,
}

/// Non-object field in a selection.
#[derive(Debug, Clone)]
pub struct SelectionTreeLeaf<S> {
    /// Name of the field.
    pub name: S,
    /// GraphQL type of the field.
    pub r#type: Type<S, Pos>,
}

/// Object field in a selection.
#[derive(Debug, Clone)]
pub struct SelectionTreeObject<S> {
    /// Name of the field.
    pub name: S,
    /// Selection for the field.
    pub selection: SelectionTree<S>,
}
