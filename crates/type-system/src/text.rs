use std::{borrow::Borrow, hash::Hash};

/// Trait that expresses owned or borrowed text.
pub trait Text: Eq + Clone + Hash + Borrow<str> {}

impl Text for &'_ str {}
