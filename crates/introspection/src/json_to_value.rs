use serde_json::Value;

use crate::error::IntrospectionError;

/// Subset of GraphQL value.
#[derive(Clone, Debug)]
pub enum GraphQLValue<Str> {
    Object(ObjectValue<Str>),
    List(ListValue<Str>),
    String(Str),
    Enum(Str),
    Boolean(bool),
    Null,
}

impl<Str> GraphQLValue<Str> {
    pub fn as_string(&self) -> Option<&Str> {
        match self {
            GraphQLValue::String(ref s) => Some(s),
            _ => None,
        }
    }
    pub fn as_enum(&self) -> Option<&Str> {
        match self {
            GraphQLValue::Enum(ref s) => Some(s),
            _ => None,
        }
    }
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            GraphQLValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_list(&self) -> Option<&ListValue<Str>> {
        match self {
            GraphQLValue::List(ref v) => Some(v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjectValue<Str> {
    pub fields: Vec<(Str, GraphQLValue<Str>)>,
}

impl<'a, Str: PartialEq<&'a str>> ObjectValue<Str> {
    /// Get field by key.
    pub fn get(&self, key: &'a str) -> Option<&GraphQLValue<Str>> {
        self.fields
            .iter()
            .find_map(|(k, v)| (*k == key).then_some(v))
    }
    /// Get field and coarse to string.
    pub fn get_str(&self, key: &'a str) -> Option<&Str> {
        self.get(key).and_then(|v| v.as_string())
    }
}

#[derive(Clone, Debug)]
pub struct ListValue<Str> {
    pub values: Vec<GraphQLValue<Str>>,
}

/// Parses JSON and converts to GraphQL value.
pub fn json_to_value(json: &str) -> Result<GraphQLValue<String>, IntrospectionError> {
    let v: Value = serde_json::from_str(json)?;

    value_to_value(v)
}

fn value_to_value(value: Value) -> Result<GraphQLValue<String>, IntrospectionError> {
    let Value::Object(mut object) = value else {
        return Err(IntrospectionError::GraphQLError("Expected an object".into()))
    };

    let Some(kind) = object.get("kind").and_then(|f| f.as_str()) else {
        return Err(IntrospectionError::GraphQLError("No string 'kind' field in given object".into()))
    };

    match kind {
        "NullValue" => Ok(GraphQLValue::Null),
        "BooleanValue" => Ok(GraphQLValue::Boolean({
            let Some(Value::Bool(value)) = object.remove("value") else {
                return Err(IntrospectionError::GraphQLError("'value' field of Boolean must be a boolean".into()));
            };

            value
        })),
        "StringValue" => Ok(GraphQLValue::String({
            let Some(Value::String(value)) = object.remove("value") else {
                return Err(IntrospectionError::GraphQLError("'value' field of StringValue must be a string".into()));
            };

            value.to_owned()
        })),
        "EnumValue" => Ok(GraphQLValue::String({
            let Some(Value::String(value)) = object.remove("value") else {
                return Err(IntrospectionError::GraphQLError("'value' field of EnumValue must be a string".into()));
            };

            value.to_owned()
        })),
        "ListValue" => {
            let Some(Value::Array(array)) = object.remove("values") else {
                return Err(IntrospectionError::GraphQLError("'values' field of a ListValue must be an array".into()));
            };
            let values = array
                .into_iter()
                .map(value_to_value)
                .collect::<Result<Vec<_>, _>>();
            let values = values?;
            Ok(GraphQLValue::List(ListValue { values }))
        }
        "ObjectValue" => {
            let Some(Value::Array(fields)) = object.remove("fields") else {
                return Err(IntrospectionError::GraphQLError("'values' field of an ObjectValue must be an array".into()));
            };
            let fields = fields
                .into_iter()
                .map(read_object_field)
                .collect::<Result<Vec<_>, _>>();
            let fields = fields?;
            Ok(GraphQLValue::Object(ObjectValue { fields }))
        }
        kind => Err(IntrospectionError::GraphQLError(format!(
            "Unknown value type '{kind}'"
        ))),
    }
}

fn read_object_field(value: Value) -> Result<(String, GraphQLValue<String>), IntrospectionError> {
    let Value::Object(mut object) = value else {
        return Err(IntrospectionError::GraphQLError("Expected an object".into()))
    };
    if object.get("kind").and_then(|v| v.as_str()) != Some("ObjectField") {
        return Err(IntrospectionError::GraphQLError(
            "Fields in an ObjectValue must be an ObjectField".into(),
        ));
    }
    let Some(Value::String(name)) = object.remove("name") else {
        return Err(IntrospectionError::GraphQLError("'values' field of an ObjectField must be an array".into()));
    };
    let Some(value) = object.remove("value") else {
        return Err(IntrospectionError::GraphQLError("'An ObjectField must have a 'value' field".into()));
    };
    let value = value_to_value(value)?;
    Ok((name, value))
}
