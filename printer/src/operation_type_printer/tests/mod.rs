use insta::assert_snapshot;

use graphql_builtins::generate_builtins;
use nitrogql_ast::{OperationDocument, TypeSystemDocument};
use nitrogql_parser::{parse_operation_document, parse_type_system_document};
use nitrogql_semantics::generate_definition_map;
use nitrogql_semantics::resolve_extensions;
use sourcemap_writer::JustWriter;

use crate::print_types_for_operation_document;
use crate::OperationTypePrinterOptions;

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
        fragment F on HasID {
            id
        }
        ",
    )
    .unwrap();
    let printed = print_document(&doc);
    assert_snapshot!(printed);
}

#[test]
fn fragment_spread_cond() {
    let doc = parse_operation_document(
        "
        query {
            me {
                id name type age
                posts {
                    id
                    ...F
                    ...P
                }
            }
        }
        fragment F on HasID {
            id
        }
        fragment P on Post {
            title
            body
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
    let schema = type_system();
    print_types_for_operation_document(
        OperationTypePrinterOptions::default(),
        &schema,
        document,
        &mut writer,
    );
    result
}
