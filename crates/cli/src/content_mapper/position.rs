use nitrogql_ast::base::Pos;

use super::protocol::PositionEncoding;

/// Converts positions in a text into offsets expected by the content mapper protocol.
pub struct SourceText<'a> {
    text: &'a str,
    encoding: PositionEncoding,
}

impl<'a> SourceText<'a> {
    pub fn new(text: &'a str, encoding: PositionEncoding) -> Self {
        SourceText { text, encoding }
    }

    /// Converts a byte offset to an offset in the specified encoding.
    pub fn encode(&self, byte_offset: usize) -> usize {
        encode_offset(self.text, byte_offset, self.encoding)
    }

    /// Returns the span (start and length, in the specified encoding) of the node at given position.
    /// Spans of nodes that cover multiple lines are cut at the end of the first line.
    /// Returns None if the position does not fit the text.
    pub fn span(&self, pos: &Pos) -> Option<(usize, usize)> {
        let node = self.text.get(pos.span())?;
        let end = pos.start + node.find('\n').unwrap_or(node.len());
        let end = pos.start + self.text[pos.start..end].trim_end_matches('\r').len();
        let encoded_start = self.encode(pos.start);
        Some((encoded_start, self.encode(end) - encoded_start))
    }

    /// Returns the span (start and length, in the specified encoding) of the node at given position,
    /// if the node is exactly given name.
    pub fn name_span(&self, pos: &Pos, name: &str) -> Option<(usize, usize)> {
        if self.text.get(pos.span())? != name {
            return None;
        }
        let encoded_start = self.encode(pos.start);
        Some((encoded_start, self.encode(pos.end) - encoded_start))
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

    /// Pos of the first occurrence of `needle` in `text`.
    fn pos_of(text: &str, needle: &str) -> Pos {
        let start = text.find(needle).unwrap();
        Pos {
            // line and column are not used
            line: 0,
            column: 0,
            start,
            end: start + needle.len(),
            file: 0,
            builtin: false,
        }
    }

    #[test]
    fn span() {
        let text = "# 😀\nquery Foo_1 { a }";
        let utf8 = SourceText::new(text, PositionEncoding::Utf8);
        assert_eq!(utf8.span(&pos_of(text, "Foo_1")), Some((13, 5)));
        let utf16 = SourceText::new(text, PositionEncoding::Utf16);
        assert_eq!(utf16.span(&pos_of(text, "Foo_1")), Some((11, 5)));
        assert_eq!(utf16.span(&pos_of(text, "😀")), Some((2, 2)));
        // empty span
        let mut pos = pos_of(text, "Foo_1");
        pos.end = pos.start;
        assert_eq!(utf16.span(&pos), Some((11, 0)));
        // out of range
        pos.end = text.len() + 1;
        assert_eq!(utf16.span(&pos), None);
    }

    #[test]
    fn multiline_span() {
        let text = "query Foo {\n  a\n}";
        let utf8 = SourceText::new(text, PositionEncoding::Utf8);
        assert_eq!(utf8.span(&pos_of(text, text)), Some((0, 11)));
        assert_eq!(utf8.span(&pos_of(text, "{\n  a\n}")), Some((10, 1)));
        let text = "query Foo {\r\n  a\r\n}";
        let utf8 = SourceText::new(text, PositionEncoding::Utf8);
        assert_eq!(utf8.span(&pos_of(text, text)), Some((0, 11)));
    }

    #[test]
    fn name_span() {
        let text = "😀 query Foo { a }";
        let utf16 = SourceText::new(text, PositionEncoding::Utf16);
        assert_eq!(utf16.name_span(&pos_of(text, "Foo"), "Foo"), Some((9, 3)));
        assert_eq!(utf16.name_span(&pos_of(text, "Foo"), "Bar"), None);
        // Pos must point exactly at the name
        assert_eq!(utf16.name_span(&pos_of(text, "query Foo"), "Foo"), None);
    }
}
