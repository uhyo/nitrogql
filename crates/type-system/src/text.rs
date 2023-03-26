use std::{
    borrow::{Borrow, Cow},
    hash::Hash,
    ops::Deref,
};

/// Trait that expresses owned or borrowed text.
pub trait Text<'a>: Eq + Clone + Hash + Borrow<str> + From<&'a str> + Deref<Target = str> {}

impl<'a> Text<'a> for &'a str {}

impl Text<'static> for String {}

impl<'a> Text<'a> for Cow<'a, str> {}
