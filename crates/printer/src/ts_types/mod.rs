use graphql_type_system::{Node, OriginalNodeRef};
use nitrogql_ast::base::{HasPos, Ident, Pos};
use sourcemap_writer::SourceMapWriter;

use super::jsdoc::print_description;

pub mod deep_merge;
mod fast_equal;
pub mod ts_types_util;
pub mod type_to_ts_type;

#[derive(Clone, Debug)]
pub enum TSType {
    /// Type variable
    TypeVariable(TypeVariable),
    /// Type Function Application
    TypeFunc(Box<TSType>, Vec<TSType>),
    /// String literal type
    StringLiteral(String),
    /// Namespace member access N.K
    NamespaceMember(String, String),
    /// Namespace member access N.K1.K2
    NamespaceMember3(String, String, String),
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
    Undefined,
    /// Null
    Null,
    /// Never
    Never,
    /// Unknown
    Unknown,
    /// Raw type (TypeScript string)
    Raw(String),
}

impl TSType {
    pub fn is_never(&self) -> bool {
        matches!(self, TSType::Never)
    }
}

#[derive(Clone, Debug)]
pub struct TypeVariable {
    name: String,
    pos: Pos,
}

impl HasPos for TypeVariable {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    fn position(&self) -> &Pos {
        &self.pos
    }
}

impl From<&Ident<'_>> for TypeVariable {
    fn from(value: &Ident) -> Self {
        TypeVariable {
            name: value.name.to_owned(),
            pos: value.position,
        }
    }
}

impl From<&str> for TypeVariable {
    fn from(value: &str) -> Self {
        TypeVariable {
            name: value.to_owned(),
            pos: Pos::builtin(),
        }
    }
}

impl<S> From<Node<&S, Pos>> for TypeVariable
where
    S: ToString,
{
    fn from(value: Node<&S, Pos>) -> Self {
        TypeVariable {
            name: value.to_string(),
            pos: *value.original_node_ref(),
        }
    }
}

// impl<S> From<S> for TypeVariable
// where
//     S: ToString,
// {
//     fn from(value: S) -> Self {
//         TypeVariable {
//             name: value.to_string(),
//             pos: Pos::builtin(),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub struct ObjectField {
    pub key: ObjectKey,
    pub r#type: TSType,
    pub readonly: bool,
    pub optional: bool,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ObjectKey {
    pub name: String,
    pub pos: Pos,
}

impl HasPos for ObjectKey {
    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
    fn position(&self) -> &Pos {
        &self.pos
    }
}

impl<'a> From<&'a Ident<'a>> for ObjectKey {
    fn from(value: &'a Ident) -> Self {
        ObjectKey {
            name: value.name.to_owned(),
            pos: value.position,
        }
    }
}

impl<'a> From<&'a str> for ObjectKey {
    fn from(value: &'a str) -> Self {
        ObjectKey {
            name: value.to_owned(),
            pos: Pos::builtin(),
        }
    }
}

impl TSType {
    /// Prints type.
    pub fn print_type(&self, writer: &mut impl SourceMapWriter) {
        match self {
            TSType::TypeVariable(ref v) => {
                writer.write_for(&v.name, v);
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
                write!(writer, "{ns}.{key}");
            }
            TSType::NamespaceMember3(ref ns, ref key1, ref key2) => {
                write!(writer, "{ns}.{key1}.{key2}");
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
                    if is_raw_ident(&field.key.name) {
                        writer.write_for(&field.key.name, &field.key);
                    } else {
                        writer.write("\"");
                        writer.write(&field.key.name);
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
            TSType::Undefined => {
                writer.write("undefined");
            }
            TSType::Never => {
                writer.write("never");
            }
            TSType::Unknown => {
                writer.write("unknown");
            }
            TSType::Raw(s) => {
                writer.write("(");
                writer.write(s);
                writer.write(")");
            }
        }
    }

    /// Converts self into a readonly type.
    /// All properties are turned into readonly properties and also array types are made readonly array types.
    pub fn into_readonly(self) -> TSType {
        match self {
            TSType::TypeFunc(func, args) => TSType::TypeFunc(
                func,
                args.into_iter().map(|ty| ty.into_readonly()).collect(),
            ),
            TSType::Array(ty) | TSType::ReadonlyArray(ty) => {
                TSType::ReadonlyArray(Box::new((*ty).into_readonly()))
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
                TSType::Intersection(types.into_iter().map(|t| t.into_readonly()).collect())
            }
            TSType::Union(types) => {
                TSType::Union(types.into_iter().map(|t| t.into_readonly()).collect())
            }
            t @ TSType::TypeVariable(_)
            | t @ TSType::StringLiteral(_)
            | t @ TSType::NamespaceMember(_, _)
            | t @ TSType::NamespaceMember3(_, _, _)
            | t @ TSType::Never
            | t @ TSType::Null
            | t @ TSType::Undefined
            | t @ TSType::Unknown
            | t @ TSType::Raw(_) => t,
        }
    }

    /// Creates an empty object type.
    pub fn empty_object() -> TSType {
        TSType::Object(vec![])
    }

    /// Creates an object type from given set of non-readonly, non-optional properties.
    pub fn object<S: Into<ObjectKey>>(
        properties: impl IntoIterator<Item = (S, TSType, Option<String>)>,
    ) -> TSType {
        TSType::Object(
            properties
                .into_iter()
                .map(|(key, ty, description)| ObjectField {
                    key: key.into(),
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
        Some(c) if !is_ascii_ident_start(c) => false,
        Some(_) => chars.all(is_ascii_ident_char),
    }
}

fn is_ascii_ident_start(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_ascii_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
}
