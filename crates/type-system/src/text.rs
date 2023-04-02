use std::{
    borrow::{Borrow, Cow},
    fmt::{Debug, Display},
    hash::Hash,
    ops::Deref,
};

/// Trait that expresses owned or borrowed text.
pub trait Text<'a>:
    PartialEq<Self>
    + PartialEq<&'a str>
    + PartialEq<String>
    + Eq
    + Clone
    + Hash
    + Borrow<str>
    + From<&'a str>
    + Deref<Target = str>
    + Debug
    + Display
{
}

impl<'a> Text<'a> for &'a str {}

impl Text<'static> for String {}

impl<'a> Text<'a> for Cow<'a, str> {}
