use super::{
    base::{HasPos, Ident, Pos},
    value::Arguments,
};

#[derive(Clone, Debug)]
pub struct Directive<'a> {
    pub position: Pos,
    /// Name of directive (does not include '@')
    pub name: Ident<'a>,
    pub arguments: Option<Arguments<'a>>,
}

impl HasPos for Directive<'_> {
    fn name(&self) -> Option<&str> {
        Some(self.name.name)
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}
