use std::{borrow::Cow, collections::HashMap};

use graphql_type_system::{NamedType, Node, RootTypes, Schema, Text};
use nitrogql_ast::{
    base::Pos,
    operation::{ExecutableDefinition, FragmentDefinition, OperationType},
    OperationDocument,
};
use nitrogql_config_file::{Config, GenerateMode};
use nitrogql_utils::clone_into;
use sourcemap_writer::SourceMapWriter;

use crate::{
    json_printer::print_to_json_string,
    operation_base_printer::{
        options::OperationBasePrinterOptions, OperationPrinterVisitor, PrintFragmentContext,
        PrintOperationContext,
    },
    ts_types::TSType,
};

use super::type_printer::{
    get_type_for_selection_set, get_type_for_variable_definitions, QueryTypePrinterContext,
};

#[derive(Clone, Debug)]
pub struct OperationTypePrinterOptions {
    pub base_options: OperationBasePrinterOptions,
    /// Whether value of variables should be printed.
    pub print_values: bool,
    /// Name of the root TypeScript namespace that contains schema types.
    pub schema_root_namespace: String,
    /// Source of schema type to import from.
    pub schema_source: String,
    /// Source of TypedDocumentNode to import from.
    pub typed_document_node_source: String,
    /// Suffix for type of variables for an operation.
    pub variables_type_suffix: String,
    /// Suffix for type of operation result.
    pub operation_result_type_suffix: String,
    /// Whether to allow undefined as input value.
    pub allow_undefined_as_optional_input: bool,
}

impl Default for OperationTypePrinterOptions {
    fn default() -> Self {
        Self {
            base_options: OperationBasePrinterOptions::default(),
            print_values: false,
            schema_root_namespace: "Schema".to_owned(),
            schema_source: "".to_owned(),
            typed_document_node_source: "@graphql-typed-document-node/core".to_owned(),
            variables_type_suffix: "Variables".to_owned(),
            operation_result_type_suffix: "Result".to_owned(),
            allow_undefined_as_optional_input: true,
        }
    }
}

impl OperationTypePrinterOptions {
    /// Generate options from config.
    pub fn from_config(config: &Config) -> Self {
        let mut result = Self {
            base_options: OperationBasePrinterOptions::from_config(config),
            ..Self::default()
        };
        if config.generate.mode == GenerateMode::StandaloneTS4_0 {
            result.print_values = true;
        }
        clone_into(
            &config.generate.name.operation_result_type_suffix,
            &mut result.operation_result_type_suffix,
        );
        clone_into(
            &config.generate.name.variables_type_suffix,
            &mut result.variables_type_suffix,
        );
        result
    }
}

pub struct OperationTypePrinterVisitor<'a, 'src> {
    options: OperationTypePrinterOptions,
    context: OperationTypePrinterContext<'a, 'src, Cow<'src, str>>,
}

impl<'a, 'src> OperationTypePrinterVisitor<'a, 'src>
where
    'a: 'src,
{
    pub fn new(
        options: OperationTypePrinterOptions,
        schema: &'a Schema<Cow<'src, str>, Pos>,
        operation: &'a OperationDocument<'src>,
    ) -> Self {
        let fragment_definitions = operation
            .definitions
            .iter()
            .filter_map(|def| match def {
                ExecutableDefinition::OperationDefinition(_) => None,
                ExecutableDefinition::FragmentDefinition(fragment_def) => {
                    Some((fragment_def.name.name, fragment_def))
                }
            })
            .collect();
        let context = OperationTypePrinterContext {
            schema,
            operation,
            fragment_definitions,
        };
        Self { options, context }
    }
}

pub struct OperationTypePrinterContext<'a, 'src, S: Text<'src>> {
    pub schema: &'a Schema<S, Pos>,
    pub operation: &'a OperationDocument<'src>,
    pub fragment_definitions: HashMap<&'src str, &'a FragmentDefinition<'src>>,
}

impl<'a, 'src> OperationPrinterVisitor for OperationTypePrinterVisitor<'a, 'src> {
    fn print_header(&self, writer: &mut impl SourceMapWriter) {
        writeln!(
            writer,
            "import type {{ TypedDocumentNode }} from \"{}\";",
            self.options.typed_document_node_source
        );
        write!(
            writer,
            "import type * as {} from \"{}\";\n\n",
            self.options.schema_root_namespace, self.options.schema_source,
        );
    }
    fn print_trailer(&self, _writer: &mut impl SourceMapWriter) {}
    fn print_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) {
        let operation = &context.operation;
        let result_type_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.operation_result_type_suffix
        );
        if context.export_result_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&result_type_name, &operation.name_pos());
        writer.write_for(" = ", &operation.selection_set);

        let root_types = self.context.schema.root_types().unwrap_or_default();
        let parent_type = select_root_type(&root_types, operation.operation_type);
        let parent_type = NamedType::from(parent_type.clone());
        let type_printer_context = QueryTypePrinterContext {
            options: &self.options,
            schema: self.context.schema,
            operation: self.context.operation,
            fragment_definitions: &self.context.fragment_definitions,
        };

        get_type_for_selection_set(
            &type_printer_context,
            &operation.selection_set,
            &parent_type,
        )
        .print_type(writer);
        writer.write(";\n\n");

        let input_variable_type = operation
            .variables_definition
            .as_ref()
            .map_or(TSType::empty_object(), |v| {
                get_type_for_variable_definitions(&type_printer_context, v)
            });
        let input_variable_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.variables_type_suffix
        );

        if context.export_input_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&input_variable_name, &operation.name_pos());
        writer.write(" = ");
        input_variable_type.print_type(writer);
        writer.write(";\n\n");

        if context.exported {
            writer.write("export ");
        } else if !self.options.print_values {
            writer.write("declare ");
        }
        writer.write("const ");
        writer.write_for(
            &context.operation_names.operation_variable_name,
            &operation.name_pos(),
        );
        writer.write_for(": ", &operation.selection_set);
        writer.write("TypedDocumentNode<");
        writer.write(&result_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        if !self.options.print_values {
            writer.write(">;\n\n");
            return;
        }
        writer.write("> = ");
        // To follow the community conventions, generated JSON has only one operation in it
        let this_document = self
            .context
            .operation
            .definitions
            .iter()
            .filter(|def| match def {
                ExecutableDefinition::FragmentDefinition(_) => true,
                ExecutableDefinition::OperationDefinition(op) => {
                    op.name.map(|ident| ident.name) == operation.name.map(|ident| ident.name)
                }
            })
            .collect::<Vec<_>>();
        writer.write(&print_to_json_string(&this_document[..]));
        // Use the `as unknown as` technique to avoid the type system complaining about
        // the type of the JSON object not matching the type of the TypedDocumentNode
        // (because of the use of enums in the TypedDocumentNode type)
        writer.write(" as unknown as TypedDocumentNode<");
        writer.write(&result_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        writer.write(">;\n\n");
    }

    fn print_fragment_definition(
        &self,
        context: PrintFragmentContext,
        writer: &mut impl SourceMapWriter,
    ) {
        let fragment = context.fragment;
        if context.exported {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(fragment.name.name, fragment);
        writer.write(" = ");

        let parent_type = NamedType::from(Node::from(
            fragment.type_condition.name,
            fragment.type_condition.position,
        ));

        let type_printer_context = QueryTypePrinterContext {
            options: &self.options,
            schema: self.context.schema,
            operation: self.context.operation,
            fragment_definitions: &self.context.fragment_definitions,
        };

        let fragment_type = get_type_for_selection_set(
            &type_printer_context,
            &fragment.selection_set,
            &parent_type,
        );
        fragment_type.print_type(writer);
        writer.write(";\n\n");
    }
    fn print_default_exported_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) {
        writer.write("export { ");
        writer.write(&context.operation_names.operation_variable_name);
        writer.write(" as default };\n\n");
    }
}

fn select_root_type<T>(root_types: &RootTypes<T>, operation_type: OperationType) -> &T {
    match operation_type {
        OperationType::Query => &root_types.query_type,
        OperationType::Mutation => &root_types.mutation_type,
        OperationType::Subscription => &root_types.subscription_type,
    }
}
