use std::{borrow::Cow, collections::HashMap};

use graphql_type_system::Schema;
use nitrogql_ast::{
    base::Pos,
    operation::OperationType,
    type_system::{
        DirectiveDefinition, SchemaDefinition, TypeDefinition, TypeSystemDefinition,
        TypeSystemDocument,
    },
};

use crate::ast_to_type_system;

#[derive(Debug)]
pub struct DefinitionMap<'a> {
    pub type_system: Schema<Cow<'a, str>, Pos>,
    pub schema: Option<&'a SchemaDefinition<'a>>,
    pub types: HashMap<&'a str, &'a TypeDefinition<'a>>,
    pub directives: HashMap<&'a str, &'a DirectiveDefinition<'a>>,
}

impl DefinitionMap<'_> {
    /// Returns a TypeDefinition for the root type of given OperationType.
    pub fn root_type(&self, op: OperationType) -> Option<&TypeDefinition> {
        let op_type_name = match self.schema {
            Some(schema) => schema
                .definitions
                .iter()
                .find(|(o, _)| *o == op)
                .map(|(_, ty)| ty.name),
            None => Some(match op {
                OperationType::Query => "Query",
                OperationType::Mutation => "Mutation",
                OperationType::Subscription => "Subscription",
            }),
        };
        let op_type_name = op_type_name?;
        self.types.get(op_type_name).cloned()
    }
}

pub fn generate_definition_map<'a, 'src>(
    document: &'a TypeSystemDocument<'src>,
) -> DefinitionMap<'src>
where
    'a: 'src,
{
    let mut result = DefinitionMap {
        type_system: ast_to_type_system(document),
        schema: None,
        types: HashMap::new(),
        directives: HashMap::new(),
    };
    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(schema) => {
                result.schema = Some(schema);
            }
            TypeSystemDefinition::TypeDefinition(def) => {
                result.types.insert(def.name().name, def);
            }
            TypeSystemDefinition::DirectiveDefinition(def) => {
                result.directives.insert(def.name.name, def);
            }
        }
    }

    result
}
