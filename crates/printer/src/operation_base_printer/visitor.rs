use std::collections::HashMap;

use nitrogql_ast::operation::{FragmentDefinition, OperationDefinition};
use sourcemap_writer::SourceMapWriter;

use super::OperationNames;

pub trait OperationPrinterVisitor {
    /// Prints header of a document.
    fn print_header(&self, writer: &mut impl SourceMapWriter);
    /// Prints trailer of a document.
    fn print_trailer(&self, writer: &mut impl SourceMapWriter);
    /// Prints one operation definition.
    fn print_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    );
    /// Prints one fragment definition.
    fn print_fragment_definition(
        &self,
        context: PrintFragmentContext,
        writer: &mut impl SourceMapWriter,
    );
    /// Prints a default export of given operation.
    fn print_default_exported_operation_definition(
        &self,
        context: PrintOperationContext,
        writer: &mut impl SourceMapWriter,
    );
}

#[derive(Copy, Clone, Debug)]
pub struct PrintOperationContext<'a> {
    /// Name of the variable for this operation.
    pub operation_names: &'a OperationNames,
    /// Whether this operation is exported.
    pub exported: bool,
    /// Whether input type of this operation is exported.
    pub export_input_type: bool,
    /// Whether result type of this operation is exported.
    pub export_result_type: bool,
    /// Operation definition.
    pub operation: &'a OperationDefinition<'a>,
    /// Map from fragment names to fragment definitions.
    pub fragments: &'a HashMap<&'a str, &'a FragmentDefinition<'a>>,
}

#[derive(Copy, Clone, Debug)]
pub struct PrintFragmentContext<'a> {
    /// Name of the variable for this fragment.
    pub var_name: &'a str,
    /// Whether this fragment is exported.
    pub exported: bool,
    /// Fragment definition.
    pub fragment: &'a FragmentDefinition<'a>,
}
