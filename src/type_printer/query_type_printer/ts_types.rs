use crate::source_map_writer::writer::SourceMapWriter;

use super::{printer::QueryTypePrinterOptions, type_printer::TypePrinter};

#[derive(Clone)]
pub enum TSType {
    /// Type variable
    TypeVariable(String),
    /// String literal type
    StringLiteral(String),
    /// Index type T[K]
    IndexType(Box<TSType>, Box<TSType>),
    /// Object type
    Object(Vec<(String, TSType)>),
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

impl TypePrinter for TSType {
    fn print_type(&self, options: &QueryTypePrinterOptions, writer: &mut impl SourceMapWriter) {
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
                parent.print_type(options, writer);
                writer.write("[");
                key.print_type(options, writer);
                writer.write("]");
            }
            TSType::Object(ref properties) => {
                if properties.is_empty() {
                    writer.write("{}");
                    return;
                }

                writer.write("{\n");
                writer.indent();
                for (key, value) in properties {
                    writer.write("\"");
                    writer.write(key);
                    writer.write("\": ");
                    value.print_type(options, writer);
                    writer.write(";\n");
                }
                writer.dedent();
                writer.write("}");
            }
            TSType::Array(ref ty) => {
                writer.write("(");
                ty.print_type(options, writer);
                writer.write(")[]");
            }
            TSType::Intersection(ref types) => {
                if types.is_empty() {
                    TSType::Unknown.print_type(options, writer);
                    return;
                }
                for (idx, ty) in types.iter().enumerate() {
                    if idx > 0 {
                        writer.write(" & ");
                    }
                    ty.print_type(options, writer);
                }
            }
            TSType::Union(ref types) => {
                if types.is_empty() {
                    TSType::Never.print_type(options, writer);
                    return;
                }
                for (idx, ty) in types.iter().enumerate() {
                    if idx > 0 {
                        writer.write(" | ");
                    }
                    ty.print_type(options, writer);
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
