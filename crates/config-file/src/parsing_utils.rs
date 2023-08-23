use std::{marker::PhantomData, str::FromStr};

use serde::{de::Visitor, Deserialize, Deserializer};

/// A string or a list of strings.
pub enum StringOrVecString {
    String(String),
    VecString(Vec<String>),
}

impl StringOrVecString {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            StringOrVecString::String(s) => vec![s],
            StringOrVecString::VecString(v) => v,
        }
    }
}

impl<'de> Deserialize<'de> for StringOrVecString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(StringOrVecStringVisitor)
    }
}

struct StringOrVecStringVisitor;

impl<'de> Visitor<'de> for StringOrVecStringVisitor {
    type Value = StringOrVecString;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or a list of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StringOrVecString::String(v.to_owned()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(StringOrVecString::String(v))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(value) = seq.next_element()? {
            vec.push(value);
        }
        Ok(StringOrVecString::VecString(vec))
    }
}

pub fn deserialize_fromstr<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
{
    deserializer.deserialize_str(FromStrVisitor(PhantomData))
}

struct FromStrVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for FromStrVisitor<T>
where
    T: FromStr,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        T::from_str(v).map_err(|_| E::custom(format!("invalid enum value: {v}")))
    }
}
