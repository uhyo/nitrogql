use pest::iterators::Pair;

use self::{
    type_definition::{build_arguments_definition, build_type_definition},
    type_extension::build_type_extension,
};

use super::{
    Rule, directives::build_directives, operation::str_to_operation_type, utils::PairExt,
    value::build_string_value,
};
use crate::parts;
use nitrogql_ast::{
    base::Ident,
    operation::OperationType,
    type_system::{
        DirectiveDefinition, SchemaDefinition, SchemaExtension, TypeSystemDefinitionOrExtension,
    },
    value::StringValue,
};

mod type_definition;
mod type_extension;

/// Builds a TypeSystemDefinitionOrExtension from a Pair for TypeSystemDefinitionOrExtension.
pub fn build_type_system_definition_or_extension(
    pair: Pair<Rule>,
    file: usize,
) -> TypeSystemDefinitionOrExtension {
    let pair = pair.only_child();
    match pair.as_rule() {
        Rule::TypeSystemDefinition => {
            let pair = pair.only_child();
            match pair.as_rule() {
                Rule::SchemaDefinition => TypeSystemDefinitionOrExtension::SchemaDefinition(
                    build_schema_definition(pair, file),
                ),
                Rule::TypeDefinition => TypeSystemDefinitionOrExtension::TypeDefinition(
                    build_type_definition(pair, file),
                ),
                Rule::DirectiveDefinition => TypeSystemDefinitionOrExtension::DirectiveDefinition(
                    build_directive_definition(pair, file),
                ),
                rule => panic!("Unexpected child of TypeSystemDefinition: {:?}", rule),
            }
        }
        Rule::TypeSystemExtension => {
            let pair = pair.only_child();
            match pair.as_rule() {
                Rule::SchemaExtension => TypeSystemDefinitionOrExtension::SchemaExtension(
                    build_schema_extension(pair, file),
                ),
                Rule::TypeExtension => {
                    TypeSystemDefinitionOrExtension::TypeExtension(build_type_extension(pair, file))
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

fn build_schema_definition(pair: Pair<Rule>, file: usize) -> SchemaDefinition {
    let position = pair.to_pos(file);
    let (description, _, directives, root_operation_type_definitions) = parts!(
        pair,
        Description opt,
        KEYWORD_schema,
        Directives opt,
        RootOperationTypeDefinitions
    );
    let definitions = build_root_operation_type_definitions(root_operation_type_definitions, file);
    SchemaDefinition {
        description: description.map(|pair| build_description(pair, file)),
        position,
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        definitions,
    }
}

fn build_directive_definition(pair: Pair<Rule>, file: usize) -> DirectiveDefinition {
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
        description: description.map(|pair| build_description(pair, file)),
        position: keyword.to_pos(file),
        name: name.to_ident(file),
        arguments: arguments.map(|pair| build_arguments_definition(pair, file)),
        repeatable: repeatable.map(|pair| pair.to_ident(file)),
        locations: {
            let locations = locations.all_children(Rule::DirectiveLocation);
            locations
                .into_iter()
                .map(|pair| pair.to_ident(file))
                .collect()
        },
        directive_keyword: keyword.to_keyword(file),
    }
}

fn build_schema_extension(pair: Pair<Rule>, file: usize) -> SchemaExtension {
    let position = pair.to_pos(file);
    let (_, _, directives, root_operation_type_definition) = parts!(
        pair,
        KEYWORD_extend,
        KEYWORD_schema,
        Directives opt,
        RootOperationTypeDefinitions opt
    );
    SchemaExtension {
        position,
        directives: directives.map_or(vec![], |pair| build_directives(pair, file)),
        definitions: root_operation_type_definition.map_or(vec![], |pair| {
            build_root_operation_type_definitions(pair, file)
        }),
    }
}

fn build_root_operation_type_definitions(
    pair: Pair<Rule>,
    file: usize,
) -> Vec<(OperationType, Ident)> {
    pair.all_children(Rule::RootOperationTypeDefinition)
        .into_iter()
        .map(|def| {
            let (operation_type, named_type) = parts!(def, OperationType, NamedType);
            (
                str_to_operation_type(operation_type.as_str()),
                named_type.to_ident(file),
            )
        })
        .collect()
}

pub fn build_description(pair: Pair<Rule>, file: usize) -> StringValue {
    let pair = pair.only_child();
    if pair.as_rule() != Rule::StringValue {
        panic!("Unexpected child of Description: {:?}", pair.as_rule())
    }
    build_string_value(pair, file)
}
