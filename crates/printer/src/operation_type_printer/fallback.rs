use std::convert::Infallible;

use nitrogql_ast::{
    OperationDocument,
    operation::{FragmentDefinition, OperationDefinition},
};
use sourcemap_writer::SourceMapWriter;

use crate::{operation_base_printer::OperationPrinter, ts_types::TSType};

use super::{
    type_source::OperationTypeSource,
    visitor::{OperationTypePrinterOptions, OperationTypePrinterVisitor},
};

/// Print a TypeScript module for given operation document, with all types being `any`.
///
/// Unlike `print_types_for_operation_document`, this does not require the document to be valid
/// against the schema. This is useful for providing the same set of exports as usual
/// for documents that have errors.
pub fn print_fallback_types_for_operation_document(
    options: OperationTypePrinterOptions,
    operation: &OperationDocument,
    writer: &mut impl SourceMapWriter,
) {
    let base_options = options.base_options.clone();
    let visitor = OperationTypePrinterVisitor::new(options, AnyTypeSource);
    let mut printer = OperationPrinter::new(base_options, visitor, writer);
    let Ok(()) = printer.print_document(operation);
}

/// Type source in which all types are `any`.
pub struct AnyTypeSource;

impl OperationTypeSource for AnyTypeSource {
    type Error = Infallible;

    fn uses_schema_types(&self) -> bool {
        false
    }

    fn operation_result_type(
        &self,
        _operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(TSType::Any)
    }

    fn operation_variables_type(
        &self,
        _operation: &OperationDefinition,
    ) -> Result<TSType, Self::Error> {
        Ok(TSType::Any)
    }

    fn fragment_type(&self, _fragment: &FragmentDefinition) -> Result<TSType, Self::Error> {
        Ok(TSType::Any)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use nitrogql_ast::set_current_file_of_pos;
    use nitrogql_parser::parse_operation_document;
    use nitrogql_semantics::resolve_operation_extensions;
    use sourcemap_writer::JustWriter;

    use super::*;

    fn print(source: &str, options: OperationTypePrinterOptions) -> String {
        set_current_file_of_pos(0);
        let doc = parse_operation_document(source).unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        print_fallback_types_for_operation_document(options, &doc, &mut writer);
        result
    }

    #[test]
    fn fallback_types() {
        // Fields do not need to exist in any schema.
        let printed = print(
            "
            query GetMe($id: ID!) {
                me(id: $id) { unknownField }
            }
            fragment F on UnknownType {
                foo
            }
            ",
            OperationTypePrinterOptions::default(),
        );
        assert_snapshot!(printed);
    }

    #[test]
    fn fallback_types_with_descriptions() {
        // Descriptions are printed in the same way as the schema-based printer.
        let printed = print(
            r#"
            "Get the current user"
            query GetMe { me { unknownField } }
            "User data"
            fragment F on UnknownType { foo }
            "#,
            OperationTypePrinterOptions::default(),
        );
        assert_snapshot!(printed);
    }

    #[test]
    fn fallback_types_named_exports() {
        let mut options = OperationTypePrinterOptions::default();
        options.base_options.default_export_for_operation = false;
        options.base_options.named_export_for_operation = true;
        options.base_options.export_result_type = true;
        options.base_options.export_input_type = true;
        let printed = print(
            "
            query A { a }
            mutation B { b }
            ",
            options,
        );
        assert_snapshot!(printed);
    }
}
