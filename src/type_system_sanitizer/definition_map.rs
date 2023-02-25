use std::collections::HashMap;

use crate::graphql_parser::ast::type_system::{
    DirectiveDefinition, SchemaDefinition, TypeDefinition,
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
