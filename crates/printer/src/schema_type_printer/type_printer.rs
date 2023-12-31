use std::{borrow::Borrow, fmt::Display};

use crate::{
    ts_types::{
        ts_types_util::ts_union, type_to_ts_type::get_ts_type_of_type, ObjectField, TSType,
    },
    utils::interface_implementers,
};
use nitrogql_ast::{
    base::{HasPos, Ident, Pos},
    type_system::{
        EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition,
        ObjectTypeDefinition, ScalarTypeDefinition, TypeDefinition, TypeSystemDefinition,
        UnionTypeDefinition,
    },
    value::StringValue,
};
use nitrogql_config_file::TypeTarget;
use sourcemap_writer::SourceMapWriter;

use crate::jsdoc::print_description as jsdoc_print_description;

use super::{
    context::SchemaTypePrinterContext,
    error::{SchemaTypePrinterError, SchemaTypePrinterResult},
};

pub trait TypePrinter {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()>;
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()>;
}

impl TypePrinter for TypeSystemDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeSystemDefinition::SchemaDefinition(_) => Ok(()),
            TypeSystemDefinition::TypeDefinition(def) => def.print_type(context, writer),
            TypeSystemDefinition::DirectiveDefinition(_) => Ok(()),
        }
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeSystemDefinition::SchemaDefinition(_) => Ok(()),
            TypeSystemDefinition::TypeDefinition(def) => def.print_representative(context, writer),
            TypeSystemDefinition::DirectiveDefinition(_) => Ok(()),
        }
    }
}

impl TypePrinter for TypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeDefinition::Scalar(def) => def.print_type(context, writer),
            TypeDefinition::Object(def) => def.print_type(context, writer),
            TypeDefinition::Interface(def) => def.print_type(context, writer),
            TypeDefinition::Union(def) => def.print_type(context, writer),
            TypeDefinition::Enum(def) => def.print_type(context, writer),
            TypeDefinition::InputObject(def) => def.print_type(context, writer),
        }
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeDefinition::Scalar(def) => def.print_representative(context, writer),
            TypeDefinition::Object(def) => def.print_representative(context, writer),
            TypeDefinition::Interface(def) => def.print_representative(context, writer),
            TypeDefinition::Union(def) => def.print_representative(context, writer),
            TypeDefinition::Enum(def) => def.print_representative(context, writer),
            TypeDefinition::InputObject(def) => def.print_representative(context, writer),
        }
    }
}

impl TypePrinter for ScalarTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let scalar_ts_type = context.scalar_types.get(self.name.name);
        let Some(scalar_type_str) = scalar_ts_type else {
            return Err(SchemaTypePrinterError::ScalarTypeNotProvided {
                position: self.position,
                name: self.name.to_string(),
            });
        };

        print_description(&self.description, writer);
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");

        export_type(
            writer,
            &self.scalar_keyword,
            &self.name,
            local_name,
            |writer| {
                writer.write(scalar_type_str.get_type(context.type_target));
            },
        );
        Ok(())
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_representative(
            writer,
            &self.scalar_keyword,
            &self.name,
            local_name,
            TypeTarget::OperationOutput,
        );
        Ok(())
    }
}

impl TypePrinter for ObjectTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        if context.type_target.is_input() {
            // Object type is not used as input type.
            return Ok(());
        }
        let type_name_ident = Ident {
            name: "__typename",
            position: Pos::builtin(),
        };
        let schema_type = context.schema.get_type(self.name.name);
        let obj_type = TSType::object(
            vec![(
                &type_name_ident,
                TSType::StringLiteral(self.name.to_string()),
                None,
            )]
            .into_iter()
            .chain(self.fields.iter().map(|field| {
                let schema_field = schema_type
                    .and_then(|ty| ty.as_object())
                    .and_then(|ty| ty.fields.iter().find(|f| f.name == field.name.name));
                (
                    &field.name,
                    get_ts_type_of_type(&field.r#type, |name| {
                        let local_name = context
                            .local_type_names
                            .get(name.name.name)
                            .expect("Local type name not generated");
                        TSType::TypeVariable(local_name.as_str().into())
                    }),
                    make_ts_description(
                        &field.description,
                        &schema_field.and_then(|f| f.deprecation.as_ref()),
                    ),
                )
            })),
        );

        print_description(&self.description, writer);
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");

        export_type(
            writer,
            &self.type_keyword,
            &self.name,
            local_name,
            |writer| {
                obj_type.print_type(writer);
            },
        );
        Ok(())
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_representative(
            writer,
            &self.type_keyword,
            &self.name,
            local_name,
            TypeTarget::OperationOutput,
        );
        Ok(())
    }
}

impl TypePrinter for InterfaceTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        if context.type_target.is_input() {
            // Interface is not used as input type.
            return Ok(());
        }
        // In generated type definitions, an interface is expressed as a union of all possible concrete types.
        let union_constituents =
            interface_implementers(context.schema, self.name.name).map(|obj| {
                TSType::TypeVariable({
                    let s: &str = (obj.name).inner_ref().borrow();
                    s.into()
                })
            });
        let intf_type = ts_union(union_constituents);

        print_description(&self.description, writer);
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_type(
            writer,
            &self.interface_keyword,
            &self.name,
            local_name,
            |writer| {
                intf_type.print_type(writer);
            },
        );
        Ok(())
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_representative(
            writer,
            &self.interface_keyword,
            &self.name,
            local_name,
            TypeTarget::OperationOutput,
        );
        Ok(())
    }
}

impl TypePrinter for UnionTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        if context.type_target.is_input() {
            // Union is not used as input type.
            return Ok(());
        }
        let union_type = ts_union(
            self.members
                .iter()
                .map(|mem| TSType::TypeVariable(mem.into())),
        );

        print_description(&self.description, writer);
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_type(
            writer,
            &self.union_keyword,
            &self.name,
            local_name,
            |writer| {
                union_type.print_type(writer);
            },
        );
        Ok(())
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_representative(
            writer,
            &self.union_keyword,
            &self.name,
            local_name,
            TypeTarget::OperationOutput,
        );
        Ok(())
    }
}

impl TypePrinter for EnumTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let enum_type = TSType::Union(
            self.values
                .iter()
                .map(|mem| TSType::StringLiteral(mem.name.to_string()))
                .collect(),
        );

        print_description(&self.description, writer);
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_type(
            writer,
            &self.enum_keyword,
            &self.name,
            local_name,
            |writer| {
                enum_type.print_type(writer);
            },
        );
        Ok(())
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_representative(
            writer,
            &self.enum_keyword,
            &self.name,
            local_name,
            TypeTarget::OperationOutput,
        );
        if context.options.emit_schema_runtime {
            writer.write_for("export const ", &self.enum_keyword);
            writer.write_for(self.name.name, &self.name);
            writer.write(" = {\n");
            writer.indent();
            for value in &self.values {
                writer.write_for(value.name.name, &value.name);
                writer.write(": \"");
                writer.write_for(value.name.name, &value.name);
                writer.write("\",\n");
            }
            writer.dedent();
            writer.write("} as const;\n")
        }
        Ok(())
    }
}

impl TypePrinter for InputObjectTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        if context.type_target.is_output() {
            // Input object is not used as output type.
            return Ok(());
        }
        let schema_type = context.schema.get_type(self.name.name);
        let obj_type = TSType::Object(
            self.fields
                .iter()
                .map(|field| {
                    let schema_field = schema_type
                        .and_then(|t| t.as_input_object())
                        .and_then(|t| t.fields.iter().find(|f| f.name == field.name.name))
                        .expect("Type system error");

                    let ts_type = get_ts_type_of_type(&field.r#type, |name| {
                        let local_name = context
                            .local_type_names
                            .get(name.name.name)
                            .expect("Local type name not generated");
                        TSType::TypeVariable(local_name.as_str().into())
                    })
                    .into_readonly();
                    let is_optional = context.options.input_nullable_field_is_optional
                        && !field.r#type.is_nonnull();
                    let ts_type = if is_optional {
                        TSType::Union(vec![ts_type, TSType::Undefined])
                    } else {
                        ts_type
                    };
                    ObjectField {
                        key: (&field.name).into(),
                        r#type: ts_type,
                        readonly: true,
                        optional: is_optional,
                        description: make_ts_description(
                            &field.description,
                            &schema_field.deprecation,
                        ),
                    }
                })
                .collect(),
        );

        print_description(&self.description, writer);
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_type(
            writer,
            &self.input_keyword,
            &self.name,
            local_name,
            |writer| {
                obj_type.print_type(writer);
            },
        );
        Ok(())
    }
    fn print_representative(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let local_name = context
            .local_type_names
            .get(self.name.name)
            .expect("Local type name not generated");
        export_representative(
            writer,
            &self.input_keyword,
            &self.name,
            local_name,
            TypeTarget::ResolverInput,
        );
        Ok(())
    }
}

fn export_type<Writer: SourceMapWriter>(
    writer: &mut Writer,
    type_keyword: &impl HasPos,
    schema_name: &Ident,
    local_name: &str,
    print_type: impl FnOnce(&mut Writer),
) {
    if schema_name.name == local_name {
        writer.write_for("export type ", type_keyword);
        writer.write_for(local_name, schema_name);
        writer.write(" = ");
        print_type(writer);
        writer.write(";\n");
    } else {
        writer.write_for("type ", type_keyword);
        writer.write_for(local_name, schema_name);
        writer.write(" = ");
        print_type(writer);
        writer.write(";\nexport type { ");
        writer.write(local_name);
        writer.write(" as ");
        writer.write(schema_name.name);
        writer.write("};\n");
    }
}

fn export_representative(
    writer: &mut impl SourceMapWriter,
    type_keyword: &impl HasPos,
    schema_name: &Ident,
    local_name: &str,
    target: TypeTarget,
) {
    if schema_name.name == local_name {
        writer.write_for("export type ", type_keyword);
        writer.write_for(local_name, schema_name);
        writeln!(writer, " = {target}.{local_name};");
    } else {
        writer.write_for("type ", type_keyword);
        writer.write_for(local_name, schema_name);
        writeln!(writer, " = {target}.{schema_name};");
        writeln!(
            writer,
            "export type {{ {local_name} as {} }};",
            schema_name.name
        );
    }
}

fn print_description(description: &Option<StringValue>, writer: &mut impl SourceMapWriter) {
    if let Some(description) = description {
        jsdoc_print_description(description, writer);
    }
}

/// Combines description and deprecation reason into a single string.
fn make_ts_description(
    description: &Option<StringValue>,
    deprecation: &Option<impl Display>,
) -> Option<String> {
    match (description, deprecation) {
        (Some(description), Some(deprecation)) => Some(format!(
            "{}\n\n@deprecated {}",
            description.value, deprecation
        )),
        (Some(description), None) => Some(description.value.clone()),
        (None, Some(deprecation)) => format!("@deprecated {}", deprecation).into(),
        (None, None) => None,
    }
}
