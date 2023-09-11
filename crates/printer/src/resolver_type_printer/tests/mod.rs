#![cfg(test)]

use insta::assert_snapshot;
use nitrogql_ast::TypeSystemDocument;
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::resolve_extensions;
use sourcemap_writer::JustWriter;

use crate::ResolverTypePrinterPlugin;

use super::{
    error::ResolverTypePrinterResult, options::ResolverTypePrinterOptions,
    printer::ResolverTypePrinter,
};

struct DummyPlugin;
impl ResolverTypePrinterPlugin for DummyPlugin {
    fn transform_resolver_output_types<'src>(
        &self,
        document: &TypeSystemDocument<'src>,
        base: std::collections::HashMap<&'src str, crate::ts_types::TSType>,
    ) -> std::collections::HashMap<&'src str, crate::ts_types::TSType> {
        unimplemented!()
    }
}

static EMPTY_PLUGINS: &[DummyPlugin] = &[];

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

            union UserOrBot = User | Bot

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
    let printed = print_document(
        &doc,
        ResolverTypePrinterOptions {
            schema_source: "schema".into(),
            ..Default::default()
        },
        EMPTY_PLUGINS,
    )
    .unwrap();
    assert_snapshot!(printed);
}

fn print_document(
    document: &TypeSystemDocument,
    options: ResolverTypePrinterOptions,
    plugins: &[impl ResolverTypePrinterPlugin],
) -> ResolverTypePrinterResult<String> {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let mut printer = ResolverTypePrinter::new(options, &mut writer);
    printer.print_document(document, plugins)?;
    Ok(result)
}
