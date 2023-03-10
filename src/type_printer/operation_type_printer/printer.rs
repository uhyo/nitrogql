use std::collections::HashMap;

use crate::{
    ast::{
        operations::{ExecutableDefinition, FragmentDefinition},
        OperationDocument, TypeSystemDocument,
    },
    checker::definition_map::DefinitionMap,
    source_map_writer::writer::SourceMapWriter,
};

use super::type_printer::TypePrinter;

pub struct SchemaRootTypes {
    pub query: String,
    pub mutation: String,
    pub subscription: String,
}

pub struct QueryTypePrinterOptions {
    /// Name of the root TypeScript namespace that contains schema types.
    pub schema_root_namespace: String,
    /// Source of schema type to import from.
    pub schema_source: String,
    /// Name of operation root types.
    pub schema_root_types: SchemaRootTypes,
    /// Suffix for type of query result.
    pub query_result_suffix: String,
    /// Suffix for type of mutation result.
    pub mutation_result_suffix: String,
    /// Suffix for type of subscription result.
    pub subscription_result_suffix: String,
    /// Suffix for type of variables for an operation.
    pub variable_type_suffix: String,
    /// Suffix for variable of query.
    pub query_variable_suffix: String,
    /// Suffix for variable of mutation.
    pub mutation_variable_suffix: String,
    /// Suffix for variable of subscription.
    pub subscription_variable_suffix: String,
    /// Source of TypedDocumentNode to import from.
    pub typed_document_node_source: String,
}

impl Default for QueryTypePrinterOptions {
    fn default() -> Self {
        QueryTypePrinterOptions {
            schema_root_namespace: "Schema".into(),
            schema_source: "".into(),
            schema_root_types: SchemaRootTypes::default(),
            query_result_suffix: "Query".into(),
            mutation_result_suffix: "Mutation".into(),
            subscription_result_suffix: "Subscription".into(),
            variable_type_suffix: "Variables".into(),
            query_variable_suffix: "Query".into(),
            mutation_variable_suffix: "Mutation".into(),
            subscription_variable_suffix: "Subscription".into(),
            typed_document_node_source: "@graphql-typed-document-node/core".into(),
        }
    }
}

impl Default for SchemaRootTypes {
    fn default() -> Self {
        SchemaRootTypes {
            query: "Query".into(),
            mutation: "Mutation".into(),
            subscription: "Subscription".into(),
        }
    }
}

pub struct QueryTypePrinterContext<'a, 'src> {
    pub options: &'a QueryTypePrinterOptions,
    pub schema: &'a TypeSystemDocument<'src>,
    pub schema_definitions: &'a DefinitionMap<'src>,
    pub fragment_definitions: &'a HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

pub struct QueryTypePrinter<'a, Writer: SourceMapWriter> {
    options: QueryTypePrinterOptions,
    writer: &'a mut Writer,
}

impl<'a, Writer> QueryTypePrinter<'a, Writer>
where
    Writer: SourceMapWriter,
{
    pub fn new(options: QueryTypePrinterOptions, writer: &'a mut Writer) -> Self {
        QueryTypePrinter { options, writer }
    }

    pub fn print_document(
        &mut self,
        document: &OperationDocument,
        schema: &TypeSystemDocument,
        definition_map: &DefinitionMap,
    ) {
        let fragment_definitions = document
            .definitions
            .iter()
            .filter_map(|def| match def {
                ExecutableDefinition::OperationDefinition(_) => None,
                ExecutableDefinition::FragmentDefinition(fragment_def) => {
                    Some((fragment_def.name.name, fragment_def))
                }
            })
            .collect();
        self.writer.write(&format!(
            "import type {{ TypedDocumentNode }} from \"{}\";\n",
            self.options.typed_document_node_source
        ));
        self.writer.write(&format!(
            "import type * as {} from \"{}\";\n\n",
            self.options.schema_root_namespace, self.options.schema_source,
        ));

        let context = QueryTypePrinterContext {
            options: &self.options,
            schema,
            schema_definitions: definition_map,
            fragment_definitions: &fragment_definitions,
        };

        document.print_type(&context, self.writer);
    }
}
