//! Utils for dealing with Pair<Rule>

use super::super::Rule;
use pest::iterators::Pair;

pub trait PairExt<'a> {
    /// Returns Pair's only child when it matches given rule. Panics otherwise.
    fn only_child(self) -> Pair<'a, Rule>;
    /// Returns whether this Pair is of given rule.
    fn is_rule(&self, rule: Rule) -> bool;
    /// Validates that all inner Pairs are of given rule and returns them.
    fn all_children(self, rule: Rule) -> Vec<Pair<'a, Rule>>;
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
            .into_iter()
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
    ($expr:expr, opt, $rule:expr) => {{
        $expr.ok()
    }};
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
        use crate::parts_mod;
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
