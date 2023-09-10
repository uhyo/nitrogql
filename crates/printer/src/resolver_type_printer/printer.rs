use std::borrow::Cow;

use graphql_type_system::Schema;
use nitrogql_ast::{base::Pos, type_system::TypeSystemDefinition, TypeSystemDocument};
use nitrogql_semantics::ast_to_type_system;
use sourcemap_writer::SourceMapWriter;

use crate::{
    resolver_type_printer::visitor::{get_resolver_type, get_ts_type_for_resolver},
    ts_types::{ObjectKey, TSType},
};

use super::{error::ResolverTypePrinterResult, options::ResolverTypePrinterOptions};

pub struct ResolverTypePrinter<'a, Writer> {
    options: ResolverTypePrinterOptions,
    writer: &'a mut Writer,
}

pub struct ResolverTypePrinterContext<'src> {
    pub options: &'src ResolverTypePrinterOptions,
    pub document: &'src TypeSystemDocument<'src>,
    pub schema: &'src Schema<Cow<'src, str>, Pos>,
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
        let schema = ast_to_type_system(document);
        let context = ResolverTypePrinterContext {
            options: &self.options,
            document,
            schema: &schema,
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
        writeln!(
            self.writer,
            "type __TypeResolver<Obj, Context, Result> = (object: Obj, context: Context, info: GraphQLResolveInfo) => Result | Promise<Result>;"
        );

        for type_definition in &document.definitions {
            if let TypeSystemDefinition::TypeDefinition(def) = type_definition {
                let ts_type = get_ts_type_for_resolver(def, &context);
                self.writer.write("type ");
                self.writer.write_for(def.name().name, def.name());
                self.writer.write(" = ");
                ts_type.print_type(self.writer);
                writeln!(self.writer, ";");
            }
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
