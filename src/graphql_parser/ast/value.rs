use super::base::{HasPos, Ident, Pos, Variable};

#[derive(Clone, Debug)]
pub enum Value<'a> {
    Variable(Variable<'a>),
    IntValue(IntValue<'a>),
    FloatValue(FloatValue<'a>),
    StringValue(StringValue<'a>),
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

#[derive(Copy, Clone, Debug)]
pub struct StringValue<'a> {
    pub position: Pos,
    // Includes quotations
    pub value: &'a str,
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
