use crate::parts;

use super::{Rule, base::build_variable, utils::PairExt};
use pest::iterators::Pair;

use nitrogql_ast::value::{
    Arguments, BooleanValue, EnumValue, FloatValue, IntValue, ListValue, NullValue, ObjectValue,
    StringValue, Value,
};

/// Builds Value from given Pair for Value.
pub fn build_value(pair: Pair<Rule>) -> Value {
    let pair = pair.only_child();
    let position = pair.to_pos();
    match pair.as_rule() {
        Rule::Variable => Value::Variable(build_variable(pair)),
        Rule::IntValue => Value::IntValue(IntValue {
            position,
            value: pair.as_str(),
        }),
        Rule::FloatValue => Value::FloatValue(FloatValue {
            position,
            value: pair.as_str(),
        }),
        Rule::StringValue => Value::StringValue(build_string_value(pair)),
        Rule::BooleanValue => {
            let keyword = pair.only_child();
            Value::BooleanValue(BooleanValue {
                position,
                keyword: keyword.as_str(),
                value: match keyword.as_rule() {
                    Rule::KEYWORD_false => false,
                    Rule::KEYWORD_true => true,
                    rule => panic!("Unexpected rule {:?} as a child of BooleanValue", rule),
                },
            })
        }
        Rule::NullValue => Value::NullValue(NullValue {
            position,
            keyword: pair.as_str(),
        }),
        Rule::EnumValue => Value::EnumValue(EnumValue {
            position,
            value: pair.as_str(),
        }),
        Rule::ListValue => {
            let values = pair
                .all_children(Rule::Value)
                .into_iter()
                .map(build_value)
                .collect();
            Value::ListValue(ListValue { position, values })
        }
        Rule::ObjectValue => {
            let fields = pair
                .all_children(Rule::ObjectField)
                .into_iter()
                .map(|field| {
                    let (name, value) = parts!(field, Name, Value);
                    (name.to_ident(), build_value(value))
                })
                .collect();
            Value::ObjectValue(ObjectValue { position, fields })
        }
        rule => panic!("Unexpected rule {:?} as a child of Value", rule),
    }
}

/// Builds Arguments from given Pair for Arguments.
pub fn build_arguments(pair: Pair<Rule>) -> Arguments {
    let position = pair.to_pos();
    Arguments {
        position,
        arguments: pair
            .all_children(Rule::Argument)
            .into_iter()
            .map(|pair| {
                let (name, value) = parts!(pair, Name, Value);
                (name.to_ident(), build_value(value))
            })
            .collect(),
    }
}

pub fn build_string_value(pair: Pair<Rule>) -> StringValue {
    let pair = pair.only_child();
    let position = pair.to_pos();
    match pair.as_rule() {
        Rule::EmptyStringValue => StringValue {
            position,
            value: String::new(),
        },
        Rule::BlockStringValue => {
            // multi line literal
            let pair_str = pair.as_str();
            let (_, end) = pair_str.split_at(3);
            let (mid, _) = end.split_at(end.len() - 3);
            StringValue {
                position,
                value: mid.into(),
            }
        }
        Rule::NormalStringValue => {
            let characters = pair.into_inner();
            let value = characters
                .map(|pair| {
                    let pair = pair.only_child();
                    match pair.as_rule() {
                        Rule::EscapedUnicodeBrace => {
                            let pair = pair.only_child();
                            let code = u32::from_str_radix(pair.as_str(), 16).unwrap();
                            char::from_u32(code).expect("Invalid character code")
                        }
                        Rule::EscapedUnicode4 => {
                            // removes \u
                            let (_, code_str) = pair.as_str().split_at(2);
                            let code = u32::from_str_radix(code_str, 16).unwrap();
                            char::from_u32(code).expect("Invalid character code")
                        }
                        Rule::EscapedCharacter => match pair.as_str() {
                            "\\\"" => '"',
                            "\\\\" => '\\',
                            "\\/" => '/',
                            "\\b" => '\u{0008}',
                            "\\f" => '\u{000c}',
                            "\\n" => '\n',
                            "\\r" => '\r',
                            "\\t" => '\t',
                            c => panic!("Unknown escape sequence '{}'", c),
                        },
                        Rule::NormalStringCharacter => pair.as_str().chars().next().unwrap(),
                        rule => panic!("Unexpected rule {:?}", rule),
                    }
                })
                .collect();
            StringValue { position, value }
        }
        rule => panic!("Unexpected rule as a child of StringValue: {:?}", rule),
    }
}
