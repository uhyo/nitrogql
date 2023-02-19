use graphql_parser::{
    query::{Field, Mutation, Query, Subscription},
    schema::Text,
    Pos,
};

pub trait HasPos {
    fn position(&self) -> Pos;
    fn name(&self) -> Option<&str>;
}

impl<'a, T: Text<'a>> HasPos for Query<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.as_ref())
    }
}

impl<'a, T: Text<'a>> HasPos for Mutation<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.as_ref())
    }
}

impl<'a, T: Text<'a>> HasPos for Subscription<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.as_ref())
    }
}

impl<'a, T: Text<'a>> HasPos for Field<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.name.as_ref())
    }
}
