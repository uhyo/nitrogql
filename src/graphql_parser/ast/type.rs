use super::base::{HasPos, Ident, Pos};

#[derive(Clone, Debug)]
pub enum Type<'a> {
    Named(NamedType<'a>),
    NonNull(Box<NonNullType<'a>>),
    List(Box<ListType<'a>>),
}

impl HasPos for Type<'_> {
    fn name(&self) -> Option<&str> {
        match self {
            Type::Named(name) => Some(name.name.name),
            Type::NonNull(_) => None,
            Type::List(_) => None,
        }
    }
    fn position(&self) -> &Pos {
        match self {
            Type::Named(name) => name.name.position(),
            Type::NonNull(non_null) => non_null.r#type.position(),
            Type::List(list) => &list.position,
        }
    }
}

impl Type<'_> {
    /// Returns a reference to the unwrapped type of self.
    pub fn unwrapped_type(&self) -> &NamedType {
        match self {
            Type::Named(name) => name,
            Type::NonNull(inner) => inner.r#type.unwrapped_type(),
            Type::List(inner) => inner.r#type.unwrapped_type(),
        }
    }
    /// Checks whether given type is the same type (invariant) as self.  
    pub fn is_same(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Named(self_name), Type::Named(other_name)) => {
                self_name.name.name == other_name.name.name
            }
            (Type::NonNull(self_inner), Type::NonNull(other_inner)) => {
                (self_inner.r#type).is_same(&other_inner.r#type)
            }
            (Type::List(self_inner), Type::List(other_inner)) => {
                self_inner.r#type.is_same(&other_inner.r#type)
            }
            _ => false,
        }
    }
    /// Returns if self is a non-nullable type.
    pub fn is_nonnull(&self) -> bool {
        matches!(self, Type::NonNull(_))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NamedType<'a> {
    pub name: Ident<'a>,
}

#[derive(Clone, Debug)]
pub struct NonNullType<'a> {
    pub r#type: Type<'a>,
}

#[derive(Clone, Debug)]
pub struct ListType<'a> {
    pub position: Pos,
    pub r#type: Type<'a>,
}
