use super::super::parser::Rule;
use pest::iterators::Pair;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pos {
    /// 0-based line
    pub line: usize,
    /// 0-base column
    pub column: usize,
    /// Flag that indicates that this Pos is not from parsed document, but is a built-in structure.
    pub builtin: bool,
}

impl Pos {
    /// Generates a built-in Pos.
    pub fn builtin() -> Self {
        Pos {
            line: 0,
            column: 0,
            builtin: true,
        }
    }
}

impl From<&Pair<'_, Rule>> for Pos {
    fn from(value: &Pair<'_, Rule>) -> Self {
        let (line, column) = value.line_col();
        // convert 1-based to0-based
        Pos {
            line: line - 1,
            column: column - 1,
            builtin: false,
        }
    }
}

/// Knows its start position.
pub trait HasPos {
    fn position(&self) -> &Pos;
    fn name(&self) -> Option<&str>;
}

/// Knows its start and end position.
pub trait HasSpan {
    fn span(&self) -> (&Pos, &Pos);
    fn name(&self) -> Option<&str>;
}

impl<T: HasSpan> HasPos for T {
    fn position(&self) -> &Pos {
        self.span().0
    }
    fn name(&self) -> Option<&str> {
        self.name()
    }
}

/// Carrier of name and pos
pub struct NamePos<'a> {
    pub name: Option<&'a str>,
    pub pos: Pos,
}

impl HasPos for NamePos<'_> {
    fn name(&self) -> Option<&str> {
        self.name
    }
    fn position(&self) -> &Pos {
        &self.pos
    }
}

/// Punctuation
#[derive(Copy, Clone, Debug)]
pub struct Punc<'a> {
    pub position: Pos,
    pub token: &'a str,
}

impl HasPos for Punc<'_> {
    fn name(&self) -> Option<&str> {
        None
    }
    fn position(&self) -> &Pos {
        &self.position
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ident<'a> {
    pub name: &'a str,
    pub position: Pos,
}

impl HasPos for Ident<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.name)
    }
}

impl<'a> From<Pair<'a, Rule>> for Ident<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        Ident {
            position: (&value).into(),
            name: value.as_str(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Variable<'a> {
    /// Variable name that does not include '$'
    pub name: &'a str,
    /// Position of '$'
    pub position: Pos,
}

impl HasPos for Variable<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.name)
    }
}
