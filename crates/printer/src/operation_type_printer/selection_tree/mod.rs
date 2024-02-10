//! Selection Tree is a tree structure that represents the selection in a GraphQL query.
//! Conversion from a selection set to a tree is basically resolution of the fragments.

use graphql_type_system::Type;
use nitrogql_ast::base::Pos;

mod to_ts;

#[derive(Debug, Clone)]
pub struct SelectionTree<S> {
    /// Name of GraphQL type that this selection corresponds to.
    pub type_name: String,
    /// Whether this selection corresponds to a list in the schema.
    pub is_list: bool,
    /// List of unaliased fields that are selected.
    pub unaliased_fields: Vec<SelectionTreeField<S>>,
    /// List of aliased fields that are selected.
    pub aliased_fields: Vec<SelectionTreeField<S>>,
}

#[derive(Debug, Clone)]
pub enum SelectionTreeField<S> {
    Leaf(SelectionTreeLeaf<S>),
    Object(SelectionTreeObject<S>),
    Branch(SelectionTreeBranch<S>),
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
    pub name: String,
    /// Selection for the field.
    pub selection: SelectionTree<S>,
}

/// Or-branch in a selection tree.
#[derive(Debug, Clone)]
pub struct SelectionTreeBranch<S> {
    /// Name of the field.
    pub name: String,
    /// List of selections for the field.
    pub selections: Vec<SelectionTree<S>>,
}
