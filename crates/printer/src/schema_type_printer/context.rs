use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use graphql_type_system::Schema;
use nitrogql_ast::{
    base::Pos,
    type_system::{TypeDefinition, TypeSystemDefinition},
    TypeSystemDocument,
};
use nitrogql_config_file::{ScalarTypeConfig, SeparateScalarTypeConfig, TypeTarget};

use crate::SchemaTypePrinterOptions;

pub struct SchemaTypePrinterContext<'src> {
    pub options: &'src SchemaTypePrinterOptions,
    pub document: &'src TypeSystemDocument<'src>,
    pub schema: &'src Schema<Cow<'src, str>, Pos>,
    // Mapping from Scalar name to TypeScript types.
    pub scalar_types: HashMap<String, ScalarTypeConfig>,
    /// Mapping from schema type name to local type name.
    pub local_type_names: HashMap<String, String>,
    /// Current output type target.
    pub type_target: TypeTarget,
}

impl SchemaTypePrinterContext<'_> {
    pub fn new<'src>(
        options: &'src SchemaTypePrinterOptions,
        document: &'src TypeSystemDocument<'src>,
        schema: &'src Schema<Cow<'src, str>, Pos>,
        type_target: TypeTarget,
    ) -> SchemaTypePrinterContext<'src> {
        let scalar_types = get_scalar_types(document, options);
        let local_type_names = make_local_type_names(document, &scalar_types);
        SchemaTypePrinterContext {
            options,
            document,
            schema,
            scalar_types,
            local_type_names,
            type_target,
        }
    }
}

// Generates TS Types for all scalars.
fn get_scalar_types(
    document: &TypeSystemDocument,
    options: &SchemaTypePrinterOptions,
) -> HashMap<String, ScalarTypeConfig> {
    document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            TypeSystemDefinition::TypeDefinition(TypeDefinition::Scalar(definition)) => {
                Some(definition)
            }
            _ => None,
        })
        .filter_map(|definition| {
            // type of scalar has two sources:
            // @nitrogql_ts_type built-in directive and scalarTypes option.
            // If scalarType is provided, it takes precedence.
            let scalar_type_from_config = options.scalar_types.get(definition.name.name);
            let directive_ts_type = definition
                .directives
                .iter()
                .find(|directive| (directive.name.name == "nitrogql_ts_type"))
                .and_then(|directive| directive.arguments.as_ref())
                .and_then(|args| {
                    let mut resolver_input_type = None;
                    let mut resolver_output_type = None;
                    let mut operation_input_type = None;
                    let mut operation_output_type = None;
                    for (key, value) in args.arguments.iter() {
                        match key.name {
                            "resolverInput" => resolver_input_type = Some(value),
                            "resolverOutput" => resolver_output_type = Some(value),
                            "operationInput" => operation_input_type = Some(value),
                            "operationOutput" => operation_output_type = Some(value),
                            _ => {}
                        }
                    }
                    if let (
                        Some(resolver_input_type),
                        Some(resolver_output_type),
                        Some(operation_input_type),
                        Some(operation_output_type),
                    ) = (
                        resolver_input_type.and_then(|v| v.as_string()),
                        resolver_output_type.and_then(|v| v.as_string()),
                        operation_input_type.and_then(|v| v.as_string()),
                        operation_output_type.and_then(|v| v.as_string()),
                    ) {
                        Some(ScalarTypeConfig::Separate(SeparateScalarTypeConfig {
                            resolver_input: resolver_input_type.value.clone(),
                            resolver_output: resolver_output_type.value.clone(),
                            operation_input: operation_input_type.value.clone(),
                            operation_output: operation_output_type.value.clone(),
                        }))
                    } else {
                        None
                    }
                });
            let scalar_ts_type = scalar_type_from_config.cloned().or(directive_ts_type);
            scalar_ts_type.map(|ty| (definition.name.name.to_owned(), ty))
        })
        .collect()
}

fn get_bag_of_identifiers(scalar_types: &HashMap<String, ScalarTypeConfig>) -> HashSet<&str> {
    let mut result = vec![];
    for value in scalar_types.values().flat_map(|v| v.type_names()) {
        let mut start_index = 0;
        let mut in_identifier = false;
        for (index, c) in value.char_indices() {
            if !in_identifier {
                if c.is_ascii_alphabetic() || c == '_' {
                    in_identifier = true;
                    start_index = index;
                }
            } else if !c.is_ascii_alphanumeric() && c != '_' {
                // end of identifier
                result.push(&value[start_index..index]);
                in_identifier = false;
            }
        }
        if in_identifier {
            result.push(&value[start_index..]);
        }
    }
    result.into_iter().collect()
}

fn make_local_type_names(
    document: &TypeSystemDocument,
    scalar_types: &HashMap<String, ScalarTypeConfig>,
) -> HashMap<String, String> {
    // The bag is the set of identifiers that appear in TypeScript types of scalars.
    // We will use this to avoid name collisions.
    let bag = get_bag_of_identifiers(scalar_types);
    document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            TypeSystemDefinition::TypeDefinition(def) => Some(def),
            _ => None,
        })
        .map(|definition| {
            let schema_name = definition.name().name;
            let local_name = if bag.contains(schema_name) {
                format!("__tmp_{schema_name}")
            } else {
                schema_name.to_owned()
            };
            (schema_name.to_owned(), local_name)
        })
        .collect()
}
