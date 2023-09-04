use nitrogql_ast::{
    type_system::{ScalarTypeDefinition, TypeDefinition, TypeSystemDefinition},
    TypeSystemDocument,
};
use sourcemap_writer::SourceMapWriter;

use super::{error::ResolverTypePrinterResult, printer::ResolverTypePrinterContext};

pub trait TypePrinter {
    fn print_type(
        &self,
        context: &ResolverTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> ResolverTypePrinterResult<()>;
}

impl TypePrinter for TypeSystemDefinition<'_> {
    fn print_type(
        &self,
        context: &ResolverTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> ResolverTypePrinterResult<()> {
        match self {
            TypeSystemDefinition::SchemaDefinition(_) => Ok(()),
            TypeSystemDefinition::TypeDefinition(type_definition) => {
                type_definition.print_type(context, writer)
            }
            TypeSystemDefinition::DirectiveDefinition(_) => Ok(()),
        }
    }
}

impl TypePrinter for TypeDefinition<'_> {
    fn print_type(
        &self,
        context: &ResolverTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> ResolverTypePrinterResult<()> {
        let name = self.name();
        write!(
            writer,
            "type {} = {}.{};\n\n",
            name, context.options.schema_root_namespace, name,
        );

        Ok(())
    }
}

impl TypePrinter for ScalarTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &ResolverTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> ResolverTypePrinterResult<()> {
        write!(
            writer,
            "type {} = {}.{};\n\n",
            self.name, context.options.schema_root_namespace, self.name,
        );

        Ok(())
    }
}
