use std::cell::Cell;

use crate::graphql_parser::parser::Rule;
use pest::iterators::Pair;

thread_local! {
    /// Current file to be used when generating Pos.
    static CURRENT_FILE_OF_POS: Cell<usize> = Cell::new(0);
}

pub fn set_current_file_of_pos(file: usize) {
    CURRENT_FILE_OF_POS.with(|cell| cell.set(file));
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Pos {
    /// 0-based line
    pub line: usize,
    /// 0-base column
    pub column: usize,
    /// file (specified by index)
    pub file: usize,
    /// Flag that indicates that this Pos is not from parsed document, but is a built-in structure.
    pub builtin: bool,
}

impl Pos {
    /// Generates a built-in Pos.
    pub fn builtin() -> Self {
        Pos {
            line: 0,
            column: 0,
            file: 0,
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
            file: CURRENT_FILE_OF_POS.with(|v| v.get()),
            builtin: false,
        }
    }
}

impl Ord for Pos {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line
            .cmp(&other.line)
            .then(self.column.cmp(&other.column))
    }
}

impl PartialOrd for Pos {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Knows its start position.
pub trait HasPos {
    fn position(&self) -> &Pos;
    fn name(&self) -> Option<&str>;
}

/// Knows its start and content.
pub trait HasSpan {
    fn position(&self) -> &Pos;
    fn name(&self) -> &str;
}

impl<T: HasSpan> HasPos for T {
    fn position(&self) -> &Pos {
        self.position()
    }
    fn name(&self) -> Option<&str> {
        Some(self.name())
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
impl<'a> From<Pair<'a, Rule>> for Punc<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        Punc {
            position: (&value).into(),
            token: value.as_str(),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Keyword<'a> {
    pub name: &'a str,
    pub position: Pos,
}

impl HasPos for Keyword<'_> {
    fn position(&self) -> &Pos {
        &self.position
    }
    fn name(&self) -> Option<&str> {
        Some(self.name)
    }
}

impl<'a> From<Pair<'a, Rule>> for Keyword<'a> {
    fn from(value: Pair<'a, Rule>) -> Self {
        Keyword {
            position: (&value).into(),
            name: value.as_str(),
        }
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
