//! This module contains AST nodes for basic components of ASTs.

use crate::current_file::get_current_file_of_pos;

/// Position in source file.
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
    /// Generates a non-built-in Pos.
    pub fn new(line: usize, column: usize) -> Self {
        Pos {
            line,
            column,
            file: get_current_file_of_pos(),
            builtin: false,
        }
    }

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

/// Punctuation token.
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

/// Keyword token.
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

/// identifier token.
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
