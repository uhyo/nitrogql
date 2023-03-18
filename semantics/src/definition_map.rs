use std::collections::HashMap;

use nitrogql_ast::{
    operation::OperationType,
    type_system::{
        DirectiveDefinition, SchemaDefinition, TypeDefinition, TypeSystemDefinition,
        TypeSystemDocument,
    },
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
    /// Returns a TypeDefinition for the root type of given OperationType.
    pub fn root_type(&self, op: OperationType) -> Option<&TypeDefinition> {
        let op_type_name = match self.schema {
            Some(ref schema) => schema
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
    let mut result = DefinitionMap::new();
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
