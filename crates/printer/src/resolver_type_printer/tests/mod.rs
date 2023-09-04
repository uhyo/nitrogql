#![cfg(test)]

use insta::assert_snapshot;
use nitrogql_ast::TypeSystemDocument;
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_extensions;
use sourcemap_writer::JustWriter;

use super::{options::ResolverTypePrinterOptions, printer::ResolverTypePrinter};

#[test]
fn resolver_printing() {
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
    let printed = print_document(&doc, Default::default());
    assert_snapshot!(printed);
}

fn print_document(document: &TypeSystemDocument, options: ResolverTypePrinterOptions) -> String {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let mut printer = ResolverTypePrinter::new(options, &mut writer);
    printer.print_document(document);
    result
}
