use crate::source_map_writer::writer::SourceMapWriter;

pub mod ts_types_util;
pub mod type_to_ts_type;

#[derive(Clone, Debug)]
pub enum TSType {
    /// Type variable
    TypeVariable(String),
    /// String literal type
    StringLiteral(String),
    /// Index type T[K]
    IndexType(Box<TSType>, Box<TSType>),
    /// Object type (key, value, readonly)
    Object(Vec<ObjectField>),
    /// Array Type
    Array(Box<TSType>),
    /// Readonly Array Type
    ReadonlyArray(Box<TSType>),
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

#[derive(Clone, Debug)]
pub struct ObjectField {
    pub key: String,
    pub r#type: TSType,
    pub readonly: bool,
    pub optional: bool,
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
                for field in properties {
                    if field.readonly {
                        writer.write("readonly ");
                    }
                    writer.write("\"");
                    writer.write(&field.key);
                    writer.write("\"");
                    if field.optional {
                        writer.write("?");
                    }
                    writer.write(": ");
                    field.r#type.print_type(writer);
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
            TSType::ReadonlyArray(ref ty) => {
                writer.write("readonly (");
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

    /// Converts self into a readonly type.
    /// All properties are turned into readonly properties and also array types are made readonly array types.
    pub fn to_readonly(self) -> TSType {
        match self {
            TSType::Array(ty) => TSType::ReadonlyArray(ty),
            TSType::Object(fields) => TSType::Object(
                fields
                    .into_iter()
                    .map(|field| ObjectField {
                        readonly: true,
                        ..field
                    })
                    .collect(),
            ),
            ty => ty,
        }
    }

    /// Creates an object type from given set of non-readonly, non-optional properties.
    pub fn object(properties: impl IntoIterator<Item = (String, TSType)>) -> TSType {
        TSType::Object(
            properties
                .into_iter()
                .map(|(key, ty)| ObjectField {
                    key,
                    r#type: ty,
                    readonly: false,
                    optional: false,
                })
                .collect(),
        )
    }
}
