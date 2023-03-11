use insta::assert_snapshot;

use crate::{
    ast::{OperationDocument, TypeSystemDocument},
    extension_resolver::resolve_extensions,
    graphql_builtins::generate_builtins,
    graphql_parser::parser::{parse_operation_document, parse_type_system_document},
    source_map_writer::just_writer::JustWriter,
};

use super::{QueryTypePrinter, QueryTypePrinterOptions};

fn type_system() -> TypeSystemDocument<'static> {
    let mut doc = parse_type_system_document(
        "
            type User implements HasID {
                id: ID!
                name: String!
                type: UserType!
                age: Int
                posts: [HasID!]!
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

            type Post implements HasID {
                id: ID!
                title: String!
                tags: [String!]
                body: String!
            }

            type Tweet implements HasId {
                id: ID!
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
    doc.extend(generate_builtins());
    let doc = resolve_extensions(doc).unwrap();
    doc
}

#[test]
fn basic_type_printing() {
    let doc = parse_operation_document(
        "
        query {
            me {
                id name type age
            }
        }
        ",
    )
    .unwrap();
    let printed = print_document(&doc);
    assert_snapshot!(printed);
}

#[test]
fn fragment_spread() {
    let doc = parse_operation_document(
        "
        query {
            me {
                id name type age
                posts {
                    id
                    ...F
                }
            }
        }
        fragment F on HasId {
            id
        }
        ",
    )
    .unwrap();
    let printed = print_document(&doc);
    assert_snapshot!(printed);
}

#[test]
fn fragment_inline_spread() {
    let doc = parse_operation_document(
        "
        query {
            me {
                id name type age
                posts {
                    ... {
                        id
                    }
                }
            }
        }
        ",
    )
    .unwrap();
    let printed = print_document(&doc);
    assert_snapshot!(printed);
}

fn print_document(document: &OperationDocument) -> String {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let mut printer = QueryTypePrinter::new(QueryTypePrinterOptions::default(), &mut writer);
    printer.print_document(document);
    result
}
