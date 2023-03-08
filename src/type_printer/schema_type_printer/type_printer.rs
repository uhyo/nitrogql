use crate::{
    graphql_parser::ast::{
        type_system::{
            EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
            ObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition, TypeSystemDefinition,
            UnionTypeDefinition,
        },
        value::StringValue,
        TypeSystemDocument,
    },
    source_map_writer::writer::SourceMapWriter,
    type_printer::ts_types::{type_to_ts_type::get_ts_type_of_type, ObjectField, TSType},
};

use crate::type_printer::jsdoc::print_description as jsdoc_print_description;

use super::{
    error::{SchemaTypePrinterError, SchemaTypePrinterResult},
    printer::SchemaTypePrinterOptions,
};

pub trait TypePrinter {
    fn print_type(
        &self,
        options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()>;
}

impl TypePrinter for TypeSystemDocument<'_> {
    fn print_type(
        &self,
        options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let schema_metadata_type = get_schema_metadata_type(self);
        writer.write("export type ");
        writer.write(&options.schema_metadata_type);
        writer.write(" = ");
        schema_metadata_type.print_type(writer);
        writer.write(";\n\n");

        for def in self.definitions.iter() {
            def.print_type(options, writer)?;
            writer.write("\n");
        }
        Ok(())
    }
}

fn get_schema_metadata_type(document: &TypeSystemDocument) -> TSType {
    let schema_definition = document.definitions.iter().find_map(|def| match def {
        TypeSystemDefinition::SchemaDefinition(def) => Some(def),
        _ => None,
    });
    if let Some(schema_def) = schema_definition {
        return TSType::object(schema_def.definitions.iter().map(|(op, ty)| {
            (
                op.as_str().to_owned(),
                TSType::TypeVariable(ty.name.to_owned()),
                schema_def.description.clone(),
            )
        }));
    }
    // If there is no schema definition, use default root type names.
    let mut operations = vec![];
    for d in document.definitions.iter() {
        let TypeSystemDefinition::TypeDefinition(ref def) = d else {
            continue;
        };
        let TypeDefinition::Object(ref def) = def else{
            continue;
        };

        match def.name.name {
            "Query" => {
                operations.push(("query".into(), def.name.name.into()));
            }
            "Mutation" => {
                operations.push(("mutation".into(), def.name.name.into()));
            }
            "Subscription" => {
                operations.push(("subscription".into(), def.name.name.into()));
            }
            _ => {}
        }
    }

    TSType::object(
        operations
            .into_iter()
            .map(|(op, ty)| (op, TSType::TypeVariable(ty), None)),
    )
}

impl TypePrinter for TypeSystemDefinition<'_> {
    fn print_type(
        &self,
        options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeSystemDefinition::SchemaDefinition(_) => Ok(()),
            TypeSystemDefinition::TypeDefinition(def) => def.print_type(options, writer),
            TypeSystemDefinition::DirectiveDefinition(_) => Ok(()),
        }
    }
}

impl TypePrinter for TypeDefinition<'_> {
    fn print_type(
        &self,
        options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeDefinition::Scalar(def) => def.print_type(options, writer),
            TypeDefinition::Object(def) => def.print_type(options, writer),
            TypeDefinition::Interface(def) => def.print_type(options, writer),
            TypeDefinition::Union(def) => def.print_type(options, writer),
            TypeDefinition::Enum(def) => def.print_type(options, writer),
            TypeDefinition::InputObject(def) => def.print_type(options, writer),
        }
    }
}

impl TypePrinter for ScalarTypeDefinition<'_> {
    fn print_type(
        &self,
        options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let Some(scalar_type_str) = options.scalar_types.get(self.name.name) else {
            return Err(SchemaTypePrinterError::ScalarTypeNotProvided {
                position: self.position,
                name: self.name.name.to_owned(),
            });
        };

        print_description(&self.description, writer);
        writer.write("export type ");
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        writer.write(&scalar_type_str);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for ObjectTypeDefinition<'_> {
    fn print_type(
        &self,
        _options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let obj_type = TSType::object(self.fields.iter().map(|field| {
            (
                field.name.name.to_owned(),
                get_ts_type_of_type(&field.r#type),
                field.description.clone(),
            )
        }));

        print_description(&self.description, writer);
        writer.write("export type ");
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        obj_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for InterfaceTypeDefinition<'_> {
    fn print_type(
        &self,
        _options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let obj_type = TSType::object(self.fields.iter().map(|field| {
            (
                field.name.name.to_owned(),
                get_ts_type_of_type(&field.r#type),
                field.description.clone(),
            )
        }));

        print_description(&self.description, writer);
        writer.write("export type ");
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        obj_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for UnionTypeDefinition<'_> {
    fn print_type(
        &self,
        _options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let union_type = TSType::Union(
            self.members
                .iter()
                .map(|mem| TSType::TypeVariable(mem.name.to_owned()))
                .collect(),
        );

        print_description(&self.description, writer);
        writer.write("export type ");
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        union_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for EnumTypeDefinition<'_> {
    fn print_type(
        &self,
        _options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let enum_type = TSType::Union(
            self.values
                .iter()
                .map(|mem| TSType::StringLiteral(mem.name.name.to_owned()))
                .collect(),
        );

        print_description(&self.description, writer);
        writer.write("export type ");
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        enum_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

impl TypePrinter for InputObjectTypeDefinition<'_> {
    fn print_type(
        &self,
        options: &SchemaTypePrinterOptions,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let obj_type = TSType::Object(
            self.fields
                .iter()
                .map(|field| {
                    let ts_type = get_ts_type_of_type(&field.r#type);
                    ObjectField {
                        key: field.name.name.to_owned(),
                        r#type: ts_type,
                        readonly: true,
                        optional: options.input_nullable_field_is_optional
                            && !field.r#type.is_nonnull(),
                        description: field.description.clone(),
                    }
                })
                .collect(),
        );

        print_description(&self.description, writer);
        writer.write("export type ");
        writer.write_for(self.name.name, &self.name);
        writer.write(" = ");
        obj_type.print_type(writer);
        writer.write(";\n");
        Ok(())
    }
}

fn print_description(description: &Option<StringValue>, writer: &mut impl SourceMapWriter) {
    if let Some(description) = description {
        jsdoc_print_description(description, writer);
    }
}
