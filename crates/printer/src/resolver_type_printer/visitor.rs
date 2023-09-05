use nitrogql_ast::type_system::{
    ArgumentsDefinition, ScalarTypeDefinition, TypeDefinition, TypeSystemDefinition,
};
use sourcemap_writer::SourceMapWriter;

use crate::ts_types::{type_to_ts_type::get_ts_type_of_type, ObjectField, ObjectKey, TSType};

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

pub fn get_resolver_type(
    def: &TypeDefinition<'_>,
    context: &ResolverTypePrinterContext,
) -> Option<TSType> {
    match def {
        TypeDefinition::Scalar(_) => None,
        TypeDefinition::Object(def) => get_object_resolver_type(def, context),
        TypeDefinition::Interface(_) => None,
        TypeDefinition::Union(_) => None,
        TypeDefinition::Enum(_) => None,
        TypeDefinition::InputObject(_) => None,
    }
}

fn get_object_resolver_type(
    def: &nitrogql_ast::type_system::ObjectTypeDefinition<'_>,
    context: &ResolverTypePrinterContext,
) -> Option<TSType> {
    let parent_type = TSType::TypeVariable((&def.name).into());
    let fields = def
        .fields
        .iter()
        .map(|field| {
            let arguments_type = field
                .arguments
                .as_ref()
                .map_or_else(|| TSType::Object(vec![]), arguments_definition_to_ts);
            let result_type = get_ts_type_of_type(&field.r#type, |name| {
                TSType::TypeVariable((&name.name).into())
            });

            let resolver_type = TSType::TypeFunc(
                Box::new(TSType::TypeVariable("__Resolver".into())),
                vec![
                    // Parent
                    parent_type.clone(),
                    // Args
                    arguments_type,
                    // Context
                    TSType::TypeVariable("Context".into()),
                    // Result
                    result_type,
                ],
            );

            ObjectField {
                key: ObjectKey::from(&field.name),
                r#type: resolver_type,
                optional: false,
                readonly: false,
                description: None,
            }
        })
        .collect();
    Some(TSType::Object(fields))
}

fn arguments_definition_to_ts(arguments: &ArgumentsDefinition) -> TSType {
    TSType::object(arguments.input_values.iter().map(|argument| {
        (
            ObjectKey::from(&argument.name),
            get_ts_type_of_type(&argument.r#type, |name| {
                TSType::TypeVariable((&name.name).into())
            }),
            argument.description.as_ref().map(|s| s.to_string()),
        )
    }))
    .into_readonly()
}
