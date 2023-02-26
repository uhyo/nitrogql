use crate::graphql_parser::ast::{r#type::Type, type_system::TypeDefinition};

use super::definition_map::DefinitionMap;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Input,
    Output,
    Both,
}

impl TypeKind {
    pub fn is_input_type(self) -> bool {
        match self {
            TypeKind::Input | TypeKind::Both => true,
            TypeKind::Output => false,
        }
    }
    pub fn is_output_type(self) -> bool {
        match self {
            TypeKind::Output | TypeKind::Both => true,
            TypeKind::Input => false,
        }
    }
}

/// classifies given type into output, input or both.
pub fn kind_of_type(definitions: &DefinitionMap, ty: &Type) -> Option<TypeKind> {
    let ty = ty.unwrapped_type();
    let ty_def = definitions.types.get(ty.name.name);
    ty_def.map(|def| match def {
        TypeDefinition::Scalar(_) => TypeKind::Both,
        TypeDefinition::Object(_) => TypeKind::Output,
        TypeDefinition::Interface(_) => TypeKind::Output,
        TypeDefinition::Union(_) => TypeKind::Output,
        TypeDefinition::Enum(_) => TypeKind::Both,
        TypeDefinition::InputObject(_) => TypeKind::Input,
    })
}

/// Checks if target type is a subtype of other type.
/// Returns None if unknown.
pub fn is_subtype(definitions: &DefinitionMap, target: &Type, other: &Type) -> Option<bool> {
    match target {
        Type::NonNull(target_inner) => {
            let other = if let Type::NonNull(other_inner) = other {
                &other_inner.r#type
            } else {
                other
            };
            return is_subtype(definitions, &target_inner.r#type, other);
        }
        Type::List(target_inner) => {
            if let Type::List(other_inner) = other {
                return is_subtype(definitions, &target_inner.r#type, &other_inner.r#type);
            } else {
                return Some(false);
            };
        }
        Type::Named(target_name) => {
            let other_name = if let Type::Named(other_name) = other {
                if target_name.name.name == other_name.name.name {
                    return Some(true);
                }
                Some(other_name)
            } else {
                None
            };
            let Some(target_def) = definitions.types.get(target_name.name.name) else {
                return None;
            };
            let other_def =
                other_name.and_then(|other_name| definitions.types.get(other_name.name.name));
            match target_def {
                TypeDefinition::Scalar(_)
                | TypeDefinition::Enum(_)
                | TypeDefinition::Union(_)
                | TypeDefinition::InputObject(_) => {
                    // These types cannot be a union member, so it can only be subtype of itself
                    return Some(false);
                }
                TypeDefinition::Interface(target_def) => {
                    // Interface type is considered a subtype of another when it explicitly implements the other
                    if let Some(ref other_name) = other_name {
                        if target_def
                            .implements
                            .iter()
                            .any(|imp| imp.name == other_name.name.name)
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
                TypeDefinition::Object(target_def) => {
                    if let Some(ref other_name) = other_name {
                        if target_def
                            .implements
                            .iter()
                            .any(|imp| imp.name == other_name.name.name)
                        {
                            return Some(true);
                        }
                    } else {
                        return Some(false);
                    }
                    if let Some(TypeDefinition::Union(ref other_def)) = other_def {
                        if other_def
                            .members
                            .iter()
                            .any(|mem| mem.name == target_name.name.name)
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
