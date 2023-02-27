use crate::source_map_writer::writer::SourceMapWriter;

use super::query_type_printer::{printer::QueryTypePrinterOptions, type_printer::TypePrinter};

pub mod ts_types_util;
pub mod type_to_ts_type;

#[derive(Clone)]
pub enum TSType {
    /// Type variable
    TypeVariable(String),
    /// String literal type
    StringLiteral(String),
    /// Index type T[K]
    IndexType(Box<TSType>, Box<TSType>),
    /// Object type (key, value, readonly)
    Object(Vec<(String, TSType, bool)>),
    /// Array Type
    Array(Box<TSType>),
    /// Union type
    Union(Vec<TSType>),
    /// Intersection type
    Intersection(Vec<TSType>),
    // /// Undefined
    // Undefined,
    /// Null
    Null,
    /// Never
    Never,
    /// Unknown
    Unknown,
}

impl TSType {
    /// Prints type.
    pub fn print_type(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TSType::TypeVariable(ref v) => {
                writer.write(v);
            }
            TSType::StringLiteral(ref v) => {
                writer.write("\"");
                writer.write(v);
                writer.write("\"");
            }
            TSType::IndexType(ref parent, ref key) => {
                parent.print_type(writer);
                writer.write("[");
                key.print_type(writer);
                writer.write("]");
            }
            TSType::Object(ref properties) => {
                if properties.is_empty() {
                    writer.write("{}");
                    return;
                }

                writer.write("{\n");
                writer.indent();
                for (key, value, readonly) in properties {
                    if *readonly {
                        writer.write("readonly ");
                    }
                    writer.write("\"");
                    writer.write(key);
                    writer.write("\": ");
                    value.print_type(writer);
                    writer.write(";\n");
                }
                writer.dedent();
                writer.write("}");
            }
            TSType::Array(ref ty) => {
                writer.write("(");
                ty.print_type(writer);
                writer.write(")[]");
            }
            TSType::Intersection(ref types) => {
                if types.is_empty() {
                    TSType::Unknown.print_type(writer);
                    return;
                }
                for (idx, ty) in types.iter().enumerate() {
                    if idx > 0 {
                        writer.write(" & ");
                    }
                    ty.print_type(writer);
                }
            }
            TSType::Union(ref types) => {
                if types.is_empty() {
                    TSType::Never.print_type(writer);
                    return;
                }
                for (idx, ty) in types.iter().enumerate() {
                    if idx > 0 {
                        writer.write(" | ");
                    }
                    ty.print_type(writer);
                }
            }
            TSType::Null => {
                writer.write("null");
            }
            TSType::Never => {
                writer.write("never");
            }
            TSType::Unknown => {
                writer.write("unknown");
            }
        }
    }
}
