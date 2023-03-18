#![cfg(test)]

use insta::assert_snapshot;

use crate::schema_type_printer::{
    error::SchemaTypePrinterResult,
    printer::{SchemaTypePrinter, SchemaTypePrinterOptions},
};
use nitrogql_ast::TypeSystemDocument;
use nitrogql_parser::parser::parse_type_system_document;
use nitrogql_semantics::resolve_extensions;
use sourcemap_writer::JustWriter;

#[test]
fn type_printing() {
    let doc = parse_type_system_document(
        "
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
    let doc = resolve_extensions(doc).unwrap();
    let printed = print_document(&doc).unwrap();
    assert_snapshot!(printed);
}

#[test]
fn type_printing_with_desc() {
    let doc = parse_type_system_document(
        r#"
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
    let doc = resolve_extensions(doc).unwrap();
    let printed = print_document(&doc).unwrap();
    assert_snapshot!(printed);
}

fn print_document(document: &TypeSystemDocument) -> SchemaTypePrinterResult<String> {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let mut printer = SchemaTypePrinter::new(SchemaTypePrinterOptions::default(), &mut writer);
    printer.print_document(document)?;
    Ok(result)
}
