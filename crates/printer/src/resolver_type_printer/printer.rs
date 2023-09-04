use nitrogql_ast::TypeSystemDocument;
use sourcemap_writer::SourceMapWriter;

use super::{
    error::ResolverTypePrinterResult, options::ResolverTypePrinterOptions, visitor::TypePrinter,
};

pub struct ResolverTypePrinter<'a, Writer> {
    options: ResolverTypePrinterOptions,
    writer: &'a mut Writer,
}

pub struct ResolverTypePrinterContext<'src> {
    pub options: &'src ResolverTypePrinterOptions,
    pub document: &'src TypeSystemDocument<'src>,
}

impl<'a, Writer> ResolverTypePrinter<'a, Writer>
where
    Writer: SourceMapWriter,
{
    pub fn new(options: ResolverTypePrinterOptions, writer: &'a mut Writer) -> Self {
        Self { options, writer }
    }

    pub fn print_document(
        &mut self,
        document: &TypeSystemDocument,
    ) -> ResolverTypePrinterResult<()> {
        let context = ResolverTypePrinterContext {
            options: &self.options,
            document,
        };

        write!(
            self.writer,
            "import type * as {} from \"{}\";\n\n",
            context.options.schema_root_namespace, context.options.schema_source,
        );

        for type_definition in &document.definitions {
            type_definition.print_type(&context, self.writer)?;
        }

        self.writer.write("export type ");
        self.writer.write(&context.options.root_resolver_type);
        self.writer.write(" = {};");
        Ok(())
    }
}
