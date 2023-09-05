use nitrogql_ast::{type_system::TypeSystemDefinition, TypeSystemDocument};
use sourcemap_writer::SourceMapWriter;

use crate::{
    resolver_type_printer::visitor::get_resolver_type,
    ts_types::{ObjectKey, TSType},
};

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

        writeln!(
            self.writer,
            "import type {{ GraphQLResolveInfo }} from \"graphql\";"
        );
        writeln!(
            self.writer,
            "import type * as {} from \"{}\";",
            context.options.schema_root_namespace, context.options.schema_source,
        );
        writeln!(
            self.writer,
            "type __Resolver<Parent, Args, Context, Result> = (parent: Parent, args: Args, context: Context, info: GraphQLResolveInfo) => Result | Promise<Result>;"
        );

        for type_definition in &document.definitions {
            type_definition.print_type(&context, self.writer)?;
        }

        let root_resolvers_type =
            TSType::object(document.definitions.iter().filter_map(|type_definition| {
                match type_definition {
                    TypeSystemDefinition::TypeDefinition(type_definition) => {
                        let resolver_type = get_resolver_type(type_definition, &context)?;
                        Some((ObjectKey::from(type_definition.name()), resolver_type, None))
                    }
                    _ => None,
                }
            }));

        write!(
            self.writer,
            "export type {}<Context> = ",
            &context.options.root_resolver_type
        );

        root_resolvers_type.print_type(self.writer);
        writeln!(self.writer, ";");

        Ok(())
    }
}
