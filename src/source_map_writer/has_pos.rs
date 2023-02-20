use graphql_parser::{
    query::{Field, FragmentDefinition, Mutation, Query, SelectionSet, Subscription},
    schema::Text,
    Pos,
};

pub trait HasPos {
    /// Position in source code (zero-based)
    fn position(&self) -> Pos;
    fn name(&self) -> Option<&str>;
}

impl<'a, T: Text<'a>> HasPos for Query<'a, T> {
    fn position(&self) -> Pos {
        // original Pos is 1-based so we need to convert
        let pos = self.position;
        Pos {
            line: pos.line - 1,
            column: pos.column - 1,
        }
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

impl<'a, T: Text<'a>> HasPos for FragmentDefinition<'a, T> {
    fn position(&self) -> Pos {
        self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.name.as_ref())
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

impl<'a, T: Text<'a>> HasPos for SelectionSet<'a, T> {
    fn position(&self) -> Pos {
        self.span.0
    }
    fn name(&self) -> Option<&str> {
        None
    }
}
