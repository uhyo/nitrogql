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
