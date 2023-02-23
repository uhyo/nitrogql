use pest::iterators::Pair;

use self::type_definition::build_type_definition;

use super::{directives::build_directives, operation::str_to_operation_type, utils::PairExt, Rule};
use crate::{
    graphql_parser::ast::{
        base::Ident,
        operations::OperationType,
        type_system::{SchemaDefinition, TypeDefinition, TypeSystemDefinitionOrExtension},
        value::StringValue,
    },
    parts,
};

mod type_definition;

/// Builds a TypeSystemDefinitionOrExtension from a Pair for TypeSystemDefinitionOrExtension.
pub fn build_type_system_definition_or_extension(
    pair: Pair<Rule>,
) -> TypeSystemDefinitionOrExtension {
    let pair = pair.only_child();
    match pair.as_rule() {
        Rule::TypeSystemDefinition => {
            let pair = pair.only_child();
            match pair.as_rule() {
                Rule::SchemaDefinition => {
                    TypeSystemDefinitionOrExtension::SchemaDefinition(build_schema_definition(pair))
                }
                Rule::TypeDefinition => {
                    TypeSystemDefinitionOrExtension::TypeDefinition(build_type_definition(pair))
                }
                Rule::DirectiveDefinition => {}
                rule => panic!("Unexpected child of TypeSystemDefinition: {:?}", rule),
            }
        }
        Rule::TypeSystemExtension => {}
        rule => panic!(
            "Unexpected child of TypeSystemDefinitionOrExtension: {:?}",
            rule
        ),
    }
}

fn build_schema_definition(pair: Pair<Rule>) -> SchemaDefinition {
    let position = (&pair).into();
    let (description, _, directives, root_operation_type_definitions) = parts!(
        pair,
        Description opt,
        KEYWORD_schema,
        Directives opt,
        RootOperationTypeDefinitions
    );
    let definitions = build_root_operation_type_definitions(root_operation_type_definitions);
    SchemaDefinition {
        position,
        directives: directives.map_or(vec![], build_directives),
        definitions,
    }
}

fn build_root_operation_type_definitions(pair: Pair<Rule>) -> Vec<(OperationType, Ident)> {
    pair.all_children(Rule::RootOperationTypeDefinition)
        .into_iter()
        .map(|def| {
            let (operation_type, named_type) = parts!(def, OperationType, NamedType);
            (
                str_to_operation_type(operation_type.as_str()),
                named_type.into(),
            )
        })
        .collect()
}

pub fn build_description(pair: Pair<Rule>) -> StringValue {
    let pair = pair.only_child();
    if pair.as_rule() != Rule::StringValue {
        panic!("Unexpected child of Description: {:?}", pair.as_rule())
    }
    let position = (&pair).into();
    StringValue {
        position,
        value: pair.as_str(),
    }
}
