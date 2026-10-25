use nitrogql_ast::base::Pos;

use super::protocol::PositionEncoding;

/// Converts positions in a text into offsets expected by the content mapper protocol.
pub struct SourceText<'a> {
    text: &'a str,
    encoding: PositionEncoding,
    /// For UTF-16, byte offsets of line starts along with their UTF-16 offsets,
    /// so that converting an offset only scans the line that contains it.
    line_starts: Vec<(usize, usize)>,
}

impl<'a> SourceText<'a> {
    pub fn new(text: &'a str, encoding: PositionEncoding) -> Self {
        let line_starts = match encoding {
            PositionEncoding::Utf8 => vec![],
            PositionEncoding::Utf16 => {
                let mut line_starts = vec![(0, 0)];
                let mut encoded = 0;
                for (offset, ch) in text.char_indices() {
                    encoded += ch.len_utf16();
                    if ch == '\n' {
                        line_starts.push((offset + 1, encoded));
                    }
                }
                line_starts
            }
        };
        SourceText {
            text,
            encoding,
            line_starts,
        }
    }

    /// Converts a byte offset to an offset in the specified encoding.
    pub fn encode(&self, byte_offset: usize) -> usize {
        match self.encoding {
            PositionEncoding::Utf8 => byte_offset,
            PositionEncoding::Utf16 => {
                let line = self
                    .line_starts
                    .partition_point(|(start, _)| *start <= byte_offset)
                    - 1;
                let (line_start, encoded) = self.line_starts[line];
                encoded + self.text[line_start..byte_offset].encode_utf16().count()
            }
        }
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
    fn encode() {
        let text = "a😀\nb\r\n\ncé😀d\n";
        let utf16 = SourceText::new(text, PositionEncoding::Utf16);
        let utf8 = SourceText::new(text, PositionEncoding::Utf8);
        for (offset, _) in text.char_indices().chain([(text.len(), ' ')]) {
            assert_eq!(utf16.encode(offset), text[..offset].encode_utf16().count());
            assert_eq!(utf8.encode(offset), offset);
        }
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
