use std::cell::RefCell;

use nitrogql_config_file::{Config, GenerateMode};
use nitrogql_utils::clone_into;
use sourcemap_writer::SourceMapWriter;

use crate::{
    jsdoc::print_description,
    operation_base_printer::{
        OperationPrinterVisitor, PrintFragmentContext, PrintOperationContext,
        options::OperationBasePrinterOptions,
    },
    operation_js_printer::{print_fragment_runtime, print_operation_runtime},
};

use super::{
    fragment_imports::{
        FRAGMENTS_EXPORT_NAME, FragmentImport, SourceNode, UTILITIES_NAMESPACE, import_namespace,
    },
    type_source::OperationTypeSource,
};
use crate::ts_types::TSType;

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
    /// Suffix for type of fragment.
    pub fragment_type_suffix: String,
    /// Whether to allow undefined as input value.
    pub allow_undefined_as_optional_input: bool,
    /// Fragments imported from other modules.
    /// Spreads of them refer to selection trees exported from those modules instead of being
    /// expanded, so that printed types do not depend on contents of other modules.
    pub fragment_imports: Vec<FragmentImport>,
    /// Whether to export selection trees of fragments, so that other modules can import them.
    pub export_fragment_trees: bool,
    /// Source of utility types for merging imported fragments to import from.
    pub fragment_utilities_source: String,
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
            fragment_type_suffix: "".to_owned(),
            allow_undefined_as_optional_input: true,
            fragment_imports: vec![],
            export_fragment_trees: false,
            fragment_utilities_source: "@nitrogql/cli/fragment-types".to_owned(),
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
        clone_into(
            &config.generate.name.fragment_type_suffix,
            &mut result.fragment_type_suffix,
        );
        result
    }
}

/// Visitor that prints declarations of operations and fragments.
/// Their types are provided by the type source.
pub struct OperationTypePrinterVisitor<'a, T> {
    options: &'a OperationTypePrinterOptions,
    types: T,
    /// Selection trees of printed fragments, exported at the end of the module.
    fragment_trees: RefCell<Vec<(String, TSType)>>,
}

impl<'a, T: OperationTypeSource> OperationTypePrinterVisitor<'a, T> {
    pub fn new(options: &'a OperationTypePrinterOptions, types: T) -> Self {
        Self {
            options,
            types,
            fragment_trees: RefCell::new(vec![]),
        }
    }

    /// Consumes the visitor and returns the type source.
    pub fn into_types(self) -> T {
        self.types
    }
}

impl<T: OperationTypeSource> OperationPrinterVisitor for OperationTypePrinterVisitor<'_, T> {
    type Error = T::Error;
    fn print_header(&self, writer: &mut impl SourceMapWriter) {
        writeln!(
            writer,
            "import type {{ TypedDocumentNode }} from \"{}\";",
            self.options.typed_document_node_source
        );
        if self.types.uses_schema_types() {
            write!(
                writer,
                "import type * as {} from \"{}\";\n\n",
                self.options.schema_root_namespace, self.options.schema_source,
            );
        } else {
            writer.write("\n");
        }
        if !self.options.fragment_imports.is_empty() || self.options.export_fragment_trees {
            writeln!(
                writer,
                "import type * as {UTILITIES_NAMESPACE} from \"{}\";",
                self.options.fragment_utilities_source
            );
            for (index, import) in self.options.fragment_imports.iter().enumerate() {
                write!(writer, "import type * as {} from ", import_namespace(index));
                let specifier = format!("\"{}\"", import.module);
                match import.source {
                    Some((ref position, ref text)) => {
                        writer.write_for(&specifier, &SourceNode { position, text })
                    }
                    None => writer.write(&specifier),
                }
                writer.write(";\n");
            }
            writer.write("\n");
        }
    }
    fn print_trailer(&self, writer: &mut impl SourceMapWriter) {
        if !self.options.export_fragment_trees {
            return;
        }
        let fields = self
            .fragment_trees
            .take()
            .into_iter()
            .map(|(name, tree)| crate::ts_types::ObjectField {
                key: name.into(),
                r#type: tree,
                description: None,
                optional: false,
                readonly: false,
            })
            .collect();
        write!(writer, "export type {FRAGMENTS_EXPORT_NAME} = ");
        TSType::Object(fields).print_type(writer);
        writer.write(";\n");
    }
    fn print_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    ) -> Result<(), Self::Error> {
        let operation = &context.operation;
        let result_type_name = format!(
            "{}{}",
            context.operation_names.operation_name, self.options.operation_result_type_suffix
        );
        if let Some(ref description) = operation.description {
            print_description(description, writer);
        }
        if context.export_result_type {
            writer.write("export ");
        }
        writer.write("type ");
        writer.write_for(&result_type_name, &operation.name_pos());
        writer.write_for(" = ", &operation.selection_set);
        self.types
            .operation_result_type(operation)?
            .print_type(writer);
        writer.write(";\n\n");

        let input_variable_type = self.types.operation_variables_type(operation)?;
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

        if let Some(ref description) = operation.description {
            print_description(description, writer);
        }
        if context.exported {
            writer.write("export ");
        }
        if !self.options.print_values {
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
            return Ok(());
        }
        writer.write("> = ");
        print_operation_runtime(writer, operation, context.fragments);
        // Use the `as unknown as` technique to avoid the type system complaining about
        // the type of the JSON object not matching the type of the TypedDocumentNode
        // (because of the use of enums in the TypedDocumentNode type)
        writer.write(" as unknown as TypedDocumentNode<");
        writer.write(&result_type_name);
        writer.write(", ");
        writer.write(&input_variable_name);
        writer.write(">;\n\n");
        Ok(())
    }

    fn print_fragment_definition(
        &self,
        context: PrintFragmentContext,
        writer: &mut impl SourceMapWriter,
    ) -> Result<(), Self::Error> {
        let fragment = context.fragment;
        // type of fragment
        if let Some(ref description) = fragment.description {
            print_description(description, writer);
        }
        if context.exported {
            writer.write("export ");
        }
        writer.write("type ");

        let fragment_type_name = format!(
            "{}{}",
            fragment.name.name, self.options.fragment_type_suffix
        );

        writer.write_for(&fragment_type_name, &fragment.name);

        let types = self
            .types
            .fragment_types(fragment, self.options.export_fragment_trees)?;
        writer.write(" = ");
        types.data.print_type(writer);
        writer.write(";\n\n");

        if let Some(tree) = types.tree {
            self.fragment_trees
                .borrow_mut()
                .push((fragment.name.name.to_owned(), tree));
        }

        // runtime value
        if let Some(ref description) = fragment.description {
            print_description(description, writer);
        }
        if context.exported {
            writer.write("export ");
        }
        if !self.options.print_values {
            writer.write("declare ");
        }
        writer.write("const ");
        writer.write_for(context.var_name, &fragment.name);
        writer.write(": ");
        writer.write("TypedDocumentNode<");
        writer.write_for(&fragment_type_name, &fragment.name);
        writer.write(", never>");
        if !self.options.print_values {
            writer.write(";\n\n");
            return Ok(());
        }
        writer.write(" = ");
        print_fragment_runtime(writer, fragment, context.fragments);
        writer.write(" as unknown as TypedDocumentNode<");
        writer.write_for(&fragment_type_name, &fragment.name);
        writer.write(", never>;\n\n");
        Ok(())
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
