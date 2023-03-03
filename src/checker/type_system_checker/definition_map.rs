use std::collections::HashMap;

use crate::graphql_parser::ast::{
    base::HasPos,
    type_system::{DirectiveDefinition, SchemaDefinition, TypeDefinition, TypeSystemDefinition},
    TypeSystemDocument,
};

#[derive(Default, Debug)]
pub struct DefinitionMap<'a> {
    pub schema: Option<&'a SchemaDefinition<'a>>,
    pub types: HashMap<&'a str, &'a TypeDefinition<'a>>,
    pub directives: HashMap<&'a str, &'a DirectiveDefinition<'a>>,
}

impl DefinitionMap<'_> {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn generate_definition_map<'a>(document: &'a TypeSystemDocument<'a>) -> DefinitionMap<'a> {
    let mut result = DefinitionMap::new();
    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(schema) => {
                result.schema = Some(schema);
            }
            TypeSystemDefinition::TypeDefinition(def) => {
                result.types.insert(
                    def.name().expect("Type definition should always have name"),
                    def,
                );
            }
            TypeSystemDefinition::DirectiveDefinition(def) => {
                result.directives.insert(def.name.name, def);
            }
        }
    }

    result
}
