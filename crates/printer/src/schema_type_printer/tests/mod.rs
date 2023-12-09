#![cfg(test)]

use insta::assert_snapshot;
use nitrogql_ast::type_system::TypeSystemDocument;

use crate::schema_type_printer::{
    error::SchemaTypePrinterResult,
    printer::{SchemaTypePrinter, SchemaTypePrinterOptions},
};
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_schema_extensions;
use sourcemap_writer::JustWriter;

#[test]
fn type_printing() {
    let doc = parse_type_system_document(
        "
            scalar ID
            scalar String
            scalar Int

            type User implements HasID {
                id: ID!
                name: String!
                type: UserType!
                age: Int
                posts: [Post!]!
            }
            interface HasID {
                id: ID!
            }
            enum UserType {
                NormalUser
                PremiumUser
            }
            type Bot implements HasID {
                id: ID!
            }

            type Post {
                id: ID!
                title: String!
                tags: [String!]
                body: String!
            }

            input UserSearchQuery {
                age: Int
                name: String
                keywords: [String!]
            }

            type Query {
                me: User!
            }
            ",
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let printed = print_document(&doc, Default::default()).unwrap();
    assert_snapshot!(printed);
}

#[test]
fn type_printing_with_desc() {
    let doc = parse_type_system_document(
        r#"
            scalar ID
            scalar String
            scalar Int

            "This is User."
            type User implements HasID {
                id: ID!
                "Name of user."
                name: String!
                type: UserType!
                "Age of user. User may choose to not register their age."
                age: Int
                posts: [Post!]!
            }
            """
            Node that has an id field.
            """
            interface HasID {
                "ID of node that is globally unique."
                id: ID!
            }
            enum UserType {
                "Stands for free plan users."
                NormalUser
                "Stands for paid plan users."
                PremiumUser
            }
            type Bot implements HasID {
                id: ID!
            }

            type Post {
                id: ID!
                title: String!
                tags: [String!]
                body: String!
            }

            input UserSearchQuery {
                age: Int
                name: String
            }

            type Query {
                """
                Returns my account.
                Note that query without authorization header results in a error.
                """
                me: User!
            }
            "#,
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let printed = print_document(&doc, Default::default()).unwrap();
    assert_snapshot!(printed);
}

#[test]
fn scalar_printing() {
    let doc = parse_type_system_document(
        "
        scalar BigInt
        scalar URL
        ",
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let mut options = SchemaTypePrinterOptions::default();
    options.scalar_types.extend(vec![
        ("BigInt".to_owned(), "bigint".to_owned()),
        ("URL".to_owned(), "string".to_owned()),
    ]);
    let printed = print_document(&doc, options).unwrap();
    assert_snapshot!(printed);
}

#[test]
fn avoid_circular_reference_1() {
    let doc = parse_type_system_document(
        "
        scalar BigInt
        scalar Date
        type Obj {
            bigint: BigInt
            date: Date!
        }
        ",
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let mut options = SchemaTypePrinterOptions::default();
    options.scalar_types.extend(vec![
        ("BigInt".to_owned(), "bigint".to_owned()),
        ("Date".to_owned(), "Date".to_owned()),
    ]);
    let printed = print_document(&doc, options).unwrap();
    // Date should be emitted as __tmp_Date
    assert_snapshot!(printed);
}

#[test]
fn avoid_circular_reference_2() {
    let doc = parse_type_system_document(
        "
        scalar BigInt
        scalar Date
        type Obj {
            bigint: BigInt
            date: Date!
        }
        ",
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let mut options = SchemaTypePrinterOptions::default();
    options.scalar_types.extend(vec![
        ("BigInt".to_owned(), "bigint".to_owned()),
        ("Date".to_owned(), "Date | string".to_owned()),
    ]);
    let printed = print_document(&doc, options).unwrap();
    // Date should be emitted as __tmp_Date
    assert_snapshot!(printed);
}

#[test]
fn avoid_circular_reference_3() {
    let doc = parse_type_system_document(
        "
        scalar Rec1
        scalar Rec2
        type Obj {
            rec1: Rec1!
            rec2: Rec2!
        }
        ",
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let mut options = SchemaTypePrinterOptions::default();
    options.scalar_types.extend(vec![
        ("Rec1".to_owned(), "string | Rec2".to_owned()),
        ("Rec2".to_owned(), "Rec1 | number".to_owned()),
    ]);
    let printed = print_document(&doc, options).unwrap();
    assert_snapshot!(printed);
}

#[test]
fn deprecated_items() {
    let doc = parse_type_system_document(
        r#"
        scalar ID
        scalar String
        scalar Int

        type User {
            id: ID!
            name: String!
            "Age of user."
            age: Int @deprecated
            gender: String @deprecated(reason: "Deprecated for political reasons")
        }

        input UserSearchQuery {
            age: Int @deprecated
            name: String
        }

        type Query {
            me: User!
        }
        "#,
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let options = SchemaTypePrinterOptions::default();
    let printed = print_document(&doc, options).unwrap();
    assert_snapshot!(printed);
}

#[test]
fn enum_runtime() {
    let doc = parse_type_system_document(
        r#"
        enum UserType {
            NormalUser
            PremiumUser
            AdminUser
        }
        "#,
    )
    .unwrap();
    let doc = resolve_schema_extensions(doc).unwrap();
    let options = SchemaTypePrinterOptions {
        emit_schema_runtime: true,
        ..SchemaTypePrinterOptions::default()
    };
    let printed = print_document(&doc, options).unwrap();
    assert_snapshot!(printed);
}

fn print_document(
    document: &TypeSystemDocument,
    options: SchemaTypePrinterOptions,
) -> SchemaTypePrinterResult<String> {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let mut printer = SchemaTypePrinter::new(options, &mut writer);
    printer.print_document(document)?;
    Ok(result)
}
