use super::{
    base::{HasPos, Ident, Pos},
    directive::Directive,
    value::Arguments,
};

#[derive(Clone, Debug)]
pub struct SelectionSet<'a> {
    pub position: Pos,
    pub selections: Vec<Selection<'a>>,
}

impl HasPos for SelectionSet<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone, Debug)]
pub enum Selection<'a> {
    Field(Field<'a>),
    FragmentSpread(FragmentSpread<'a>),
    InlineFragment(InlineFragment<'a>),
}

impl<'a> Selection<'a> {
    pub fn directives(&self) -> &[Directive<'a>] {
        match self {
            Selection::Field(field) => &field.directives,
            Selection::FragmentSpread(fragment_spread) => &fragment_spread.directives,
            Selection::InlineFragment(inline_fragment) => &inline_fragment.directives,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Field<'a> {
    pub alias: Option<Ident<'a>>,
    pub name: Ident<'a>,
    pub arguments: Option<Arguments<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: Option<SelectionSet<'a>>,
}

#[derive(Clone, Debug)]
pub struct FragmentSpread<'a> {
    pub position: Pos,
    pub fragment_name: Ident<'a>,
    pub directives: Vec<Directive<'a>>,
}

#[derive(Clone, Debug)]
pub struct InlineFragment<'a> {
    pub position: Pos,
    pub type_condition: Option<Ident<'a>>,
    pub directives: Vec<Directive<'a>>,
    pub selection_set: SelectionSet<'a>,
}
