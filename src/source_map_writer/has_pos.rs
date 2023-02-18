use graphql_parser::{
    query::{Field, Query},
    schema::Text,
    Pos,
};

pub trait HasPos {
    fn position(&self) -> Pos;
}

impl<'a, T: Text<'a>> HasPos for Query<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
}

impl<'a, T: Text<'a>> HasPos for Field<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
}
