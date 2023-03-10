use crate::{ast::value::StringValue, source_map_writer::writer::SourceMapWriter};

use super::jsdoc::print_description;

pub mod ts_types_util;
pub mod type_to_ts_type;

#[derive(Clone, Debug)]
pub enum TSType {
    /// Type variable
    TypeVariable(String),
    /// Type Function Application
    TypeFunc(Box<TSType>, Vec<TSType>),
    /// String literal type
    StringLiteral(String),
    /// Namespace member access N.K
    NamespaceMember(String, String),
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
    pub description: Option<StringValue>,
}

impl TSType {
    /// Prints type.
    pub fn print_type(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TSType::TypeVariable(ref v) => {
                writer.write(v);
            }
            TSType::TypeFunc(ref f, ref args) => {
                f.print_type(writer);
                writer.write("<");
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        writer.write(", ");
                    }
                    arg.print_type(writer);
                }
                writer.write(">");
            }
            TSType::StringLiteral(ref v) => {
                writer.write("\"");
                writer.write(v);
                writer.write("\"");
            }
            TSType::NamespaceMember(ref ns, ref key) => {
                writer.write(ns);
                writer.write(".");
                writer.write(key);
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
                    if let Some(ref description) = field.description {
                        print_description(description, writer);
                    }
                    if field.readonly {
                        writer.write("readonly ");
                    }
                    if is_raw_ident(&field.key) {
                        writer.write(&field.key);
                    } else {
                        writer.write("\"");
                        writer.write(&field.key);
                        writer.write("\"");
                    }
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
            TSType::TypeFunc(func, args) => TSType::TypeFunc(
                func.to_owned(),
                args.into_iter().map(|ty| ty.to_readonly()).collect(),
            ),
            TSType::Array(ty) | TSType::ReadonlyArray(ty) => {
                TSType::ReadonlyArray(Box::new((*ty).to_readonly()))
            }
            TSType::Object(fields) => TSType::Object(
                fields
                    .into_iter()
                    .map(|field| ObjectField {
                        readonly: true,
                        ..field
                    })
                    .collect(),
            ),
            TSType::Intersection(types) => {
                TSType::Intersection(types.into_iter().map(|t| t.to_readonly()).collect())
            }
            TSType::Union(types) => {
                TSType::Union(types.into_iter().map(|t| t.to_readonly()).collect())
            }
            TSType::IndexType(t1, t2) => {
                TSType::IndexType(Box::new((*t1).to_readonly()), Box::new((*t2).to_readonly()))
            }
            t @ TSType::TypeVariable(_)
            | t @ TSType::StringLiteral(_)
            | t @ TSType::NamespaceMember(_, _)
            | t @ TSType::Never
            | t @ TSType::Null
            | t @ TSType::Unknown => t,
        }
    }

    /// Creates an object type from given set of non-readonly, non-optional properties.
    pub fn object(
        properties: impl IntoIterator<Item = (String, TSType, Option<StringValue>)>,
    ) -> TSType {
        TSType::Object(
            properties
                .into_iter()
                .map(|(key, ty, description)| ObjectField {
                    key,
                    r#type: ty,
                    readonly: false,
                    optional: false,
                    description,
                })
                .collect(),
        )
    }
}

/// Returns true if given key can be printed as object property without quotations.
fn is_raw_ident(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        None => false,
        Some(c) if !c.is_ascii_alphabetic() => false,
        Some(_) => chars.all(|c| c.is_ascii_alphanumeric()),
    }
}
