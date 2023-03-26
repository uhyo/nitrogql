use std::{fmt::Display, ops::Deref};

use crate::variable::Variable;

use super::base::{HasPos, Ident, Pos};

/// A GraphQL Value.
#[derive(Clone, Debug)]
pub enum Value<'a> {
    Variable(Variable<'a>),
    IntValue(IntValue<'a>),
    FloatValue(FloatValue<'a>),
    StringValue(StringValue),
    BooleanValue(BooleanValue<'a>),
    NullValue(NullValue<'a>),
    EnumValue(EnumValue<'a>),
    ListValue(ListValue<'a>),
    ObjectValue(ObjectValue<'a>),
}

impl HasPos for Value<'_> {
    fn name(&self) -> Option<&str> {
        match self {
            Value::Variable(v) => Some(v.name),
            Value::EnumValue(v) => Some(v.value),
            _ => None,
        }
    }
    fn position(&self) -> &Pos {
        match self {
            Value::Variable(v) => v.position(),
            Value::BooleanValue(v) => &v.position,
            Value::IntValue(v) => &v.position,
            Value::FloatValue(v) => &v.position,
            Value::StringValue(v) => &v.position,
            Value::NullValue(v) => &v.position,
            Value::EnumValue(v) => &v.position,
            Value::ListValue(v) => &v.position,
            Value::ObjectValue(v) => &v.position,
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::BooleanValue(b) => {
                if b.value {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Value::IntValue(i) => write!(f, "{}", i.value),
            Value::FloatValue(i) => write!(f, "{}", i.value),
            // TODO: escaping not implemented for ease
            Value::StringValue(i) => write!(f, "\"{}\"", i.value),
            Value::EnumValue(i) => write!(f, "{}", i.value),
            Value::NullValue(_) => write!(f, "null"),
            Value::Variable(v) => write!(f, "${}", v.name),
            Value::ListValue(l) => {
                write!(f, "[")?;
                for (idx, v) in l.values.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::ObjectValue(l) => {
                write!(f, "{{")?;
                for (idx, (key, value)) in l.fields.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}: {}", key.name, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct IntValue<'a> {
    pub position: Pos,
    pub value: &'a str,
}

#[derive(Copy, Clone, Debug)]
pub struct FloatValue<'a> {
    pub position: Pos,
    pub value: &'a str,
}

#[derive(Clone, Debug)]
pub struct StringValue {
    pub position: Pos,
    /// Parsed value of string literal
    pub value: String,
}

impl Deref for StringValue {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BooleanValue<'a> {
    pub position: Pos,
    pub keyword: &'a str,
    pub value: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct NullValue<'a> {
    pub position: Pos,
    pub keyword: &'a str,
}

#[derive(Copy, Clone, Debug)]
pub struct EnumValue<'a> {
    pub position: Pos,
    pub value: &'a str,
}

#[derive(Clone, Debug)]
pub struct ListValue<'a> {
    pub position: Pos,
    pub values: Vec<Value<'a>>,
}

#[derive(Clone, Debug)]
pub struct ObjectValue<'a> {
    pub position: Pos,
    pub fields: Vec<(Ident<'a>, Value<'a>)>,
}

#[derive(Clone, Debug)]
pub struct Arguments<'a> {
    pub position: Pos,
    pub arguments: Vec<(Ident<'a>, Value<'a>)>,
}
