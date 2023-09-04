use nitrogql_ast::TypeSystemDocument;
use sourcemap_writer::SourceMapWriter;

use super::options::ResolverTypePrinterOptions;

pub struct ResolverTypePrinter<'a, Writer> {
    options: ResolverTypePrinterOptions,
    writer: &'a mut Writer,
}

impl<'a, Writer> ResolverTypePrinter<'a, Writer>
where
    Writer: SourceMapWriter,
{
    pub fn new(options: ResolverTypePrinterOptions, writer: &'a mut Writer) -> Self {
        Self { options, writer }
    }

    pub fn print_document(&mut self, document: &TypeSystemDocument) {
        self.writer.write("export type ");
        self.writer.write(&self.options.root_resolver_type);
        self.writer.write(" = {};");
    }
}
