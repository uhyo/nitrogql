use super::{
    base::{Ident, Pos},
    value::Arguments,
};

#[derive(Clone, Debug)]
pub struct Directive<'a> {
    pub position: Pos,
    /// Name of directive (does not include '@')
    pub name: Ident<'a>,
    pub arguments: Option<Arguments<'a>>,
}
