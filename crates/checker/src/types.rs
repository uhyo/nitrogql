use graphql_type_system::{Schema, Text, Type, TypeDefinition};
use nitrogql_ast::base::Pos;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TypeInOutKind {
    Input,
    Output,
    Both,
}

impl TypeInOutKind {
    pub fn is_input_type(self) -> bool {
        match self {
            TypeInOutKind::Input | TypeInOutKind::Both => true,
            TypeInOutKind::Output => false,
        }
    }
    pub fn is_output_type(self) -> bool {
        match self {
            TypeInOutKind::Output | TypeInOutKind::Both => true,
            TypeInOutKind::Input => false,
        }
    }
}

/// classifies given type into output, input or both.
pub fn inout_kind_of_type<'src, S: Text<'src>>(
    definitions: &Schema<S, Pos>,
    type_name: &str,
) -> Option<TypeInOutKind> {
    let ty_def = definitions.get_type(type_name);
    ty_def.map(|def| match **def {
        TypeDefinition::Scalar(_) => TypeInOutKind::Both,
        TypeDefinition::Object(_) => TypeInOutKind::Output,
        TypeDefinition::Interface(_) => TypeInOutKind::Output,
        TypeDefinition::Union(_) => TypeInOutKind::Output,
        TypeDefinition::Enum(_) => TypeInOutKind::Both,
        TypeDefinition::InputObject(_) => TypeInOutKind::Input,
    })
}

/// Checks if target type is a subtype of other type.
/// Returns None if unknown.
pub fn is_subtype<'src, S: Text<'src>>(
    definitions: &Schema<S, Pos>,
    target: &Type<S, Pos>,
    other: &Type<S, Pos>,
) -> Option<bool> {
    match target {
        Type::NonNull(target_inner) => {
            let other = if let Type::NonNull(other_inner) = other {
                other_inner.as_inner()
            } else {
                other
            };
            return is_subtype(definitions, target_inner.as_inner(), other);
        }
        Type::List(target_inner) => {
            if let Type::List(other_inner) = other {
                return is_subtype(definitions, target_inner.as_inner(), other_inner.as_inner());
            } else {
                return Some(false);
            };
        }
        Type::Named(target_name) => {
            let other_name = if let Type::Named(other_name) = other {
                if **target_name == ***other_name {
                    return Some(true);
                }
                Some(other_name)
            } else {
                None
            };
            let Some(target_def) = definitions.get_type(&target_name) else {
                return None;
            };
            let other_def = other_name.and_then(|other_name| definitions.get_type(&other_name));
            match **target_def {
                TypeDefinition::Scalar(_)
                | TypeDefinition::Enum(_)
                | TypeDefinition::Union(_)
                | TypeDefinition::InputObject(_) => {
                    // These types cannot be a union member, so it can only be subtype of itself
                    return Some(false);
                }
                TypeDefinition::Interface(ref target_def) => {
                    // Interface type is considered a subtype of another when it explicitly implements the other
                    if let Some(other_name) = other_name {
                        if target_def
                            .interfaces
                            .iter()
                            .any(|imp| imp == &***other_name)
                        {
                            return Some(true);
                        } else {
                            if other_def.is_some() {
                                return Some(false);
                            } else {
                                return None;
                            }
                        }
                    } else {
                        return Some(false);
                    }
                }
                TypeDefinition::Object(ref target_def) => {
                    if let Some(other_name) = other_name {
                        if target_def
                            .interfaces
                            .iter()
                            .any(|imp| imp == &***other_name)
                        {
                            return Some(true);
                        }
                    } else {
                        return Some(false);
                    }
                    if let Some(other_def) = other_def.and_then(|def| def.as_union()) {
                        if other_def
                            .possible_types
                            .iter()
                            .any(|mem| mem == &***target_name)
                        {
                            return Some(true);
                        }
                    }
                    if other_def.is_some() {
                        return Some(false);
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}
