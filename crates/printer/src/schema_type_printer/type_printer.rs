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
        TypeSystemDocument, UnionTypeDefinition,
    },
    value::StringValue,
};
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
}

impl TypePrinter for TypeSystemDocument<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        let schema_metadata_type = get_schema_metadata_type(self);
        writer.write("export type ");
        writer.write(&context.options.schema_metadata_type);
        writer.write(" = ");
        schema_metadata_type.print_type(writer);
        writer.write(";\n\n");
        // Print utility types
        writer.write(
            "type __Beautify<Obj> = { [K in keyof Obj]: Obj[K] } & {};
export type __SelectionSet<Orig, Obj, Others> =
  __Beautify<Pick<{
    [K in keyof Orig]: Obj extends { [P in K]?: infer V } ? V : unknown
  }, Extract<keyof Orig, keyof Obj>> & Others>;
",
        );

        for def in self.definitions.iter() {
            def.print_type(context, writer)?;
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
                op.as_str(),
                TSType::TypeVariable(ty.into()),
                schema_def.description.as_ref().map(|d| d.value.clone()),
            )
        }));
    }
    // If there is no schema definition, use default root type names.
    let mut operations = vec![];
    for d in document.definitions.iter() {
        let TypeSystemDefinition::TypeDefinition(ref def) = d else {
            continue;
        };
        let TypeDefinition::Object(ref def) = def else {
            continue;
        };

        match def.name.name {
            "Query" => {
                operations.push(("query", (&def.name).into()));
            }
            "Mutation" => {
                operations.push(("mutation", (&def.name).into()));
            }
            "Subscription" => {
                operations.push(("subscription", (&def.name).into()));
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
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
        match self {
            TypeSystemDefinition::SchemaDefinition(_) => Ok(()),
            TypeSystemDefinition::TypeDefinition(def) => def.print_type(context, writer),
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
                // TODO: vary on target type
                writer.write(scalar_type_str.as_operation_output_type());
            },
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
}

impl TypePrinter for InterfaceTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
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
}

impl TypePrinter for UnionTypeDefinition<'_> {
    fn print_type(
        &self,
        context: &SchemaTypePrinterContext,
        writer: &mut impl SourceMapWriter,
    ) -> SchemaTypePrinterResult<()> {
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

        if context.options.emit_schema_runtime {
            writer.write_for("export const ", &self.enum_keyword);
            writer.write_for(self.name.name, &self.name);
            writer.write(" = {\n");
            for value in &self.values {
                writer.write_for(value.name.name, &value.name);
                writer.write(": \"");
                writer.write_for(value.name.name, &value.name);
                writer.write("\",\n");
            }
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
