//! Utils for dealing with Pair<Rule>

use super::super::Rule;
use nitrogql_ast::base::{Ident, Keyword, Pos};
use pest::iterators::Pair;

pub trait PairExt<'a> {
    /// Returns Pair's only child when it matches given rule. Panics otherwise.
    fn only_child(self) -> Pair<'a, Rule>;
    /// Returns whether this Pair is of given rule.
    fn is_rule(&self, rule: Rule) -> bool;
    /// Validates that all inner Pairs are of given rule and returns them.
    fn all_children(self, rule: Rule) -> Vec<Pair<'a, Rule>>;
    /// Generate a Pos for this pair. `file` is the index of the file being parsed.
    fn to_pos(&self, file: usize) -> Pos;
    // /// Generate a Punc from this pair.
    // fn to_punc(&self) -> Punc<'a>;
    /// Generate a Keyword from this pair.
    fn to_keyword(&self, file: usize) -> Keyword<'a>;
    /// Generate an Ident from this pair.
    fn to_ident(&self, file: usize) -> Ident<'a>;
}

impl<'a> PairExt<'a> for Pair<'a, Rule> {
    fn only_child(self) -> Pair<'a, Rule> {
        let self_rule = self.as_rule();
        let mut children = self.into_inner();
        let Some(fst) = children.next() else {
            panic!("Expected 1 child of {:?}, actual 0", self_rule)
        };
        let None = children.next() else {
            panic!("Expected 1 child for {:?}, actual 2 or more", self_rule)
        };
        fst
    }
    fn is_rule(&self, rule: Rule) -> bool {
        self.as_rule() == rule
    }
    /// Validates that all inner Pairs are of given rule and returns them.
    fn all_children(self, rule: Rule) -> Vec<Pair<'a, Rule>> {
        self.into_inner()
            .filter(|pair| {
                if !pair.is_rule(rule) {
                    panic!(
                        "Expected a child of {:?}, actual {:?}",
                        rule,
                        pair.as_rule()
                    );
                }
                true
            })
            .collect()
    }

    fn to_pos(&self, file: usize) -> Pos {
        let (line, column) = self.line_col();
        let span = self.as_span();
        // A pair can end with ignored tokens (e.g. when an optional part at the end is absent),
        // which are not part of the node.
        let end = span.start() + trim_ignored_end(span.as_str());
        // convert 1-based to 0-based
        Pos::new(line - 1, column - 1, span.start()..end, file)
    }
    // fn to_punc(&self) -> Punc<'a> {
    //     Punc {
    //         position: self.to_pos(),
    //         token: self.as_str(),
    //     }
    // }
    fn to_keyword(&self, file: usize) -> Keyword<'a> {
        Keyword {
            position: self.to_pos(file),
            name: self.as_str(),
        }
    }
    fn to_ident(&self, file: usize) -> Ident<'a> {
        Ident {
            position: self.to_pos(file),
            name: self.as_str(),
        }
    }
}

fn is_ignored_char(c: char) -> bool {
    matches!(c, '\u{FEFF}' | '\t' | ' ' | '\n' | '\r' | ',')
}

/// Returns the length of given text without trailing ignored tokens
/// (whitespace, commas and comments).
/// `text` must be a sequence of tokens and ignored tokens.
/// `#import` statements are treated as comments too; they only appear at the top level.
fn trim_ignored_end(text: &str) -> usize {
    // Comments end with a newline, so text can only end with ignored tokens
    // if it ends with an ignored character.
    if !text.ends_with(is_ignored_char) {
        return text.len();
    }
    let mut end = 0;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let offset = text.len() - rest.len();
        let token_len = if is_ignored_char(c) {
            rest = &rest[c.len_utf8()..];
            continue;
        } else if c == '#' {
            rest = &rest[rest.find('\n').map_or(rest.len(), |idx| idx + 1)..];
            continue;
        } else if let Some(content) = rest.strip_prefix("\"\"\"") {
            // block string; `\"""` is an escaped delimiter
            let mut len = 3;
            let mut content = content;
            loop {
                match content.find("\"\"\"") {
                    Some(idx) if content[..idx].ends_with('\\') => {
                        len += idx + 3;
                        content = &content[idx + 3..];
                    }
                    Some(idx) => break len + idx + 3,
                    None => break rest.len(),
                }
            }
        } else if c == '"' {
            let mut chars = rest.char_indices().skip(1);
            loop {
                match chars.next() {
                    Some((_, '\\')) => {
                        chars.next();
                    }
                    Some((idx, '"')) => break idx + 1,
                    Some(_) => {}
                    None => break rest.len(),
                }
            }
        } else {
            c.len_utf8()
        };
        end = offset + token_len;
        rest = &rest[token_len..];
    }
    end
}

#[macro_export]
macro_rules! parts_mod {
    ($expr:expr, , $rule:expr) => {
        match $expr {
            Ok(pair) => pair,
            Err(Some(rule)) => panic!("Expected {:?}, actual {:?}", $rule, rule),
            Err(None) => panic!("Expected {:?}, actual nothing", $rule),
        }
    };
    ($expr:expr, opt, $rule:expr) => {{ $expr.ok() }};
}

#[macro_export]
macro_rules! parts {
    (
        $pair:expr,
        $(
            $rule:ident
            $($ident:ident)?
        ),*
    ) => {{
        use $crate::parts_mod;
        let mut pairs = $pair.into_inner().into_iter().peekable();
        (
            $(
                parts_mod!(
                    {
                        let rule = Rule::$rule;
                        let next_pair = pairs.peek();
                        match next_pair {
                            None => Err(None),
                            Some(pair) if pair.is_rule(rule) => {
                                Ok(pairs.next().unwrap())
                            },
                            Some(pair) => {
                                Err(Some(pair.as_rule()))
                            }
                        }
                    },
                    $($ident)?,
                    Rule::$rule
                )
            ),*
        )
    }};
}

#[cfg(test)]
mod tests {
    use super::trim_ignored_end;

    fn trimmed(text: &str) -> &str {
        &text[..trim_ignored_end(text)]
    }

    #[test]
    fn trims_ignored_tokens() {
        assert_eq!(trimmed("foo"), "foo");
        assert_eq!(trimmed("foo bar \n\t,\r\n"), "foo bar");
        assert_eq!(trimmed("foo # comment\n  # another\n "), "foo");
        assert_eq!(trimmed("foo\u{FEFF} "), "foo");
        assert_eq!(trimmed(" \n"), "");
    }

    #[test]
    fn keeps_strings() {
        assert_eq!(trimmed("foo: \"a # b \" \n"), "foo: \"a # b \"");
        assert_eq!(trimmed("foo: \"a \\\" b\" \n"), "foo: \"a \\\" b\"");
        assert_eq!(trimmed("foo: \"\" \n"), "foo: \"\"");
        assert_eq!(
            trimmed("\"\"\"\n # a \\\"\"\" b \n\"\"\"\n"),
            "\"\"\"\n # a \\\"\"\" b \n\"\"\""
        );
    }
}
