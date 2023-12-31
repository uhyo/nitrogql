use std::{borrow::Cow, collections::HashMap};

use graphql_type_system::Schema;
use nitrogql_ast::{
    base::Pos,
    type_system::{TypeDefinition, TypeSystemDefinition},
    TypeSystemDocument,
};
use nitrogql_semantics::ast_to_type_system;
use sourcemap_writer::SourceMapWriter;

use crate::{
    resolver_type_printer::visitor::{get_resolver_type, get_ts_type_for_resolver_output},
    ts_types::{ts_types_util::ts_union, ObjectField, ObjectKey, TSType},
};

use super::{
    error::ResolverTypePrinterResult, options::ResolverTypePrinterOptions,
    plugin::ResolverTypePrinterPlugin,
};

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
        plugins: &[impl ResolverTypePrinterPlugin],
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

        let ts_types: HashMap<&str, TSType> = document
            .definitions
            .iter()
            .filter_map(|type_definition| match type_definition {
                TypeSystemDefinition::TypeDefinition(type_definition) => {
                    let resolver_type = get_ts_type_for_resolver_output(type_definition, &context);
                    Some((type_definition.name().name, resolver_type))
                }
                _ => None,
            })
            .collect();
        let ts_types = plugins.iter().fold(ts_types, |acc, plugin| {
            plugin.transform_resolver_output_types(document, &self.options, acc)
        });

        let document_for_resolvers = plugins.iter().fold(Cow::Borrowed(document), |acc, plugin| {
            match plugin.transform_document_for_resolvers(acc.as_ref()) {
                Some(next) => Cow::Owned(next),
                None => acc,
            }
        });

        // Emit each schema type (resolver output variant) as a local type alias.
        // This helps users to read generated types.
        for type_definition in &document_for_resolvers.definitions {
            if let TypeSystemDefinition::TypeDefinition(def) = type_definition {
                if matches!(def, TypeDefinition::InputObject(_)) {
                    // input types can never be resolver outputs.
                    continue;
                }
                let ts_type = ts_types.get(def.name().name).unwrap();

                self.writer.write("type ");
                self.writer.write_for(def.name().name, def.name());
                self.writer.write(" = ");
                ts_type.print_type(self.writer);
                writeln!(self.writer, ";");
            }
        }

        let root_resolvers_type = TSType::Object(
            document_for_resolvers
                .definitions
                .iter()
                .filter_map(|type_definition| match type_definition {
                    TypeSystemDefinition::TypeDefinition(type_definition) => {
                        let resolver_type = get_resolver_type(type_definition, &context)?;
                        let optional = is_empty_object(&resolver_type);
                        Some(ObjectField {
                            key: ObjectKey::from(type_definition.name()),
                            r#type: resolver_type,
                            description: None,
                            readonly: false,
                            optional,
                        })
                    }
                    _ => None,
                })
                .collect(),
        );

        let type_names_type = ts_union(document_for_resolvers.definitions.iter().filter_map(
            |type_definition| match type_definition {
                TypeSystemDefinition::TypeDefinition(type_definition)
                    if !matches!(type_definition, TypeDefinition::InputObject(_)) =>
                {
                    Some(TSType::StringLiteral(type_definition.name().to_string()))
                }
                _ => None,
            },
        ));
        let resolver_output_type = TSType::Object(
            document_for_resolvers
                .definitions
                .iter()
                .filter_map(|type_definition| match type_definition {
                    TypeSystemDefinition::TypeDefinition(type_definition)
                        if !matches!(type_definition, TypeDefinition::InputObject(_)) =>
                    {
                        Some(ObjectField {
                            key: ObjectKey::from(type_definition.name()),
                            r#type: TSType::TypeVariable(type_definition.name().into()),
                            description: None,
                            readonly: false,
                            optional: false,
                        })
                    }
                    _ => None,
                })
                .collect(),
        );

        write!(
            self.writer,
            "export type {}<Context> = ",
            &context.options.root_resolver_type
        );
        root_resolvers_type.print_type(self.writer);
        writeln!(self.writer, ";");

        write!(
            self.writer,
            "export type {}<T extends ",
            &context.options.resolver_output_type
        );
        type_names_type.print_type(self.writer);
        writeln!(self.writer, "> = ");
        resolver_output_type.print_type(self.writer);
        writeln!(self.writer, "[T];");

        Ok(())
    }
}

fn is_empty_object(ty: &TSType) -> bool {
    if let TSType::Object(fields) = ty {
        fields.is_empty()
    } else {
        false
    }
}
