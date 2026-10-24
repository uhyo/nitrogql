use nitrogql_ast::base::Pos;

use super::protocol::PositionEncoding;

/// Converts positions in a text into offsets expected by the content mapper protocol.
pub struct LineIndex<'a> {
    text: &'a str,
    /// Byte offset of the start of each line.
    line_starts: Vec<usize>,
    encoding: PositionEncoding,
}

impl<'a> LineIndex<'a> {
    pub fn new(text: &'a str, encoding: PositionEncoding) -> Self {
        let line_starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(idx, _)| idx + 1))
            .collect();
        LineIndex {
            text,
            line_starts,
            encoding,
        }
    }

    /// Converts a 0-based line and column (counted in characters) to a byte offset.
    /// Returns None if the position is out of range.
    pub fn byte_offset(&self, line: usize, column: usize) -> Option<usize> {
        let line_start = *self.line_starts.get(line)?;
        let line_text = &self.text[line_start..];
        let line_text = &line_text[..line_text.find('\n').unwrap_or(line_text.len())];
        if column == 0 {
            return Some(line_start);
        }
        match line_text.char_indices().nth(column) {
            Some((idx, _)) => Some(line_start + idx),
            // column right after the last character
            None if line_text.chars().count() == column => Some(line_start + line_text.len()),
            None => None,
        }
    }

    /// Converts a byte offset to an offset in the specified encoding.
    pub fn encode(&self, byte_offset: usize) -> usize {
        encode_offset(self.text, byte_offset, self.encoding)
    }

    /// Calculates the span (start and length, in the specified encoding)
    /// of an identifier-like token starting at given position.
    /// Length is 0 if no such token is found.
    pub fn token_span(&self, pos: &Pos) -> Option<(usize, usize)> {
        let start = self.byte_offset(pos.line, pos.column)?;
        let len = self.text[start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(self.text.len() - start);
        let encoded_start = self.encode(start);
        Some((encoded_start, self.encode(start + len) - encoded_start))
    }

    /// Calculates the span (start and length, in the specified encoding)
    /// of given name located at given position.
    /// Since position of some nodes points to a preceding keyword (e.g. `fragment Foo`),
    /// the name is searched for in the rest of the line.
    /// Returns None if the name is not found.
    pub fn name_span(&self, pos: &Pos, name: &str) -> Option<(usize, usize)> {
        let pos_offset = self.byte_offset(pos.line, pos.column)?;
        let rest_of_line = &self.text[pos_offset..];
        let rest_of_line = &rest_of_line[..rest_of_line.find('\n').unwrap_or(rest_of_line.len())];
        let is_word_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
        let start = pos_offset
            + rest_of_line
                .match_indices(name)
                .map(|(idx, _)| idx)
                .find(|&idx| {
                    let before = rest_of_line[..idx].chars().next_back();
                    let after = rest_of_line[idx + name.len()..].chars().next();
                    !before.is_some_and(is_word_char) && !after.is_some_and(is_word_char)
                })?;
        let encoded_start = self.encode(start);
        Some((
            encoded_start,
            self.encode(start + name.len()) - encoded_start,
        ))
    }
}

/// Converts a byte offset in given text to an offset in the specified encoding.
pub fn encode_offset(text: &str, byte_offset: usize, encoding: PositionEncoding) -> usize {
    match encoding {
        PositionEncoding::Utf8 => byte_offset,
        PositionEncoding::Utf16 => text[..byte_offset].encode_utf16().count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(line: usize, column: usize) -> Pos {
        Pos {
            line,
            column,
            file: 0,
            builtin: false,
        }
    }

    #[test]
    fn byte_offset() {
        let index = LineIndex::new("abc\n😀de\n", PositionEncoding::Utf8);
        assert_eq!(index.byte_offset(0, 0), Some(0));
        assert_eq!(index.byte_offset(0, 3), Some(3));
        assert_eq!(index.byte_offset(1, 1), Some(8));
        assert_eq!(index.byte_offset(1, 3), Some(10));
        assert_eq!(index.byte_offset(1, 4), None);
        assert_eq!(index.byte_offset(2, 0), Some(11));
        assert_eq!(index.byte_offset(3, 0), None);
    }

    #[test]
    fn token_span() {
        let text = "# 😀\nquery Foo_1 { a }";
        let utf8 = LineIndex::new(text, PositionEncoding::Utf8);
        assert_eq!(utf8.token_span(&pos(1, 6)), Some((13, 5)));
        let utf16 = LineIndex::new(text, PositionEncoding::Utf16);
        assert_eq!(utf16.token_span(&pos(1, 6)), Some((11, 5)));
        assert_eq!(utf16.token_span(&pos(1, 5)), Some((10, 0)));
    }

    #[test]
    fn name_span() {
        let text = "😀 query Foo { a }";
        let utf16 = LineIndex::new(text, PositionEncoding::Utf16);
        assert_eq!(utf16.name_span(&pos(0, 8), "Foo"), Some((9, 3)));
        assert_eq!(utf16.name_span(&pos(0, 8), "Bar"), None);
        // name after a keyword
        assert_eq!(utf16.name_span(&pos(0, 2), "Foo"), Some((9, 3)));
        // only whole words match
        let text = "fragment FooBar on Foo";
        let utf8 = LineIndex::new(text, PositionEncoding::Utf8);
        assert_eq!(utf8.name_span(&pos(0, 0), "Foo"), Some((19, 3)));
        assert_eq!(utf8.name_span(&pos(0, 0), "Bar"), None);
    }
}
