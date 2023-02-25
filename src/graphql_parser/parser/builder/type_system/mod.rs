use pest::iterators::Pair;

use self::{
    type_definition::{build_arguments_definition, build_type_definition},
    type_extension::build_type_extension,
};

use super::{
    directives::build_directives, operation::str_to_operation_type, utils::PairExt,
    value::build_string_value, Rule,
};
use crate::{
    graphql_parser::ast::{
        base::Ident,
        operations::OperationType,
        type_system::{
            DirectiveDefinition, SchemaDefinition, SchemaExtension, TypeDefinition,
            TypeSystemDefinitionOrExtension,
        },
        value::StringValue,
    },
    parts,
};

mod type_definition;
mod type_extension;

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
                Rule::DirectiveDefinition => TypeSystemDefinitionOrExtension::DirectiveDefinition(
                    build_directive_definition(pair),
                ),
                rule => panic!("Unexpected child of TypeSystemDefinition: {:?}", rule),
            }
        }
        Rule::TypeSystemExtension => {
            let pair = pair.only_child();
            match pair.as_rule() {
                Rule::SchemaExtension => {
                    TypeSystemDefinitionOrExtension::SchemaExtension(build_schema_extension(pair))
                }
                Rule::TypeExtension => {
                    TypeSystemDefinitionOrExtension::TypeExtension(build_type_extension(pair))
                }
                rule => panic!("Unexpected child of TypeSystemExtension: {:?}", rule),
            }
        }
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
        description: description.map(build_description),
        position,
        directives: directives.map_or(vec![], build_directives),
        definitions,
    }
}

fn build_directive_definition(pair: Pair<Rule>) -> DirectiveDefinition {
    let (description, keyword, name, arguments, repeatable, _, locations) = parts!(
        pair,
        Description opt,
        KEYWORD_directive,
        Name,
        ArgumentsDefinition opt,
        KEYWORD_repeatable opt,
        KEYWORD_on,
        DirectiveLocations
    );
    DirectiveDefinition {
        description: description.map(build_description),
        position: (&keyword).into(),
        name: name.into(),
        arguments: arguments.map(build_arguments_definition),
        repeatable: repeatable.map(|pair| pair.into()),
        locations: {
            let locations = locations.all_children(Rule::DirectiveLocation);
            locations.into_iter().map(|pair| pair.into()).collect()
        },
    }
}

fn build_schema_extension(pair: Pair<Rule>) -> SchemaExtension {
    let position = (&pair).into();
    let (_, _, directives, root_operation_type_definition) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_schema,
        Directives opt,
        RootOperationTypeDefinitions opt
    );
    SchemaExtension {
        position,
        directives: directives.map_or(vec![], build_directives),
        definitions: root_operation_type_definition
            .map_or(vec![], build_root_operation_type_definitions),
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
    build_string_value(pair)
}
