#![cfg(test)]

use insta::assert_snapshot;
use nitrogql_ast::OperationDocument;
use nitrogql_parser::parse_operation_document;
use nitrogql_semantics::resolve_operation_extensions;
use sourcemap_writer::JustWriter;

use crate::{print_js_for_operation_document, OperationJSPrinterOptions};

#[test]
fn print_query() {
    let document = parse(
        r#"
        query MyQuery {
            user {
                id
                name
            }
        }
    "#,
    );

    assert_snapshot!(print_js(&document));
}

#[test]
fn print_fragment_only() {
    let document = parse(
        r#"
        fragment Foo on User {
            id name
        }
    "#,
    );

    assert_snapshot!(print_js(&document));
}

#[test]
fn print_fragment_and_query() {
    let document = parse(
        r#"
        query MyQuery {
            user {
                ...Foo
            }
        }

        fragment Foo on User {
            id name
        }
    "#,
    );

    assert_snapshot!(print_js(&document))
}

#[test]
fn print_query_with_variables() {
    let document = parse(
        r#"
        query MyQuery($id: ID!) {
            user(id: $id) {
                id
                name
            }
        }
    "#,
    );

    assert_snapshot!(print_js(&document));
}

#[test]
fn print_nested_fragments() {
    let document = parse(
        r#"
        query MyQuery {
            user {
                ...Foo
            }
        }

        fragment Foo on User {
            id
            ...Bar
        }

        fragment Bar on User {
            name
        }
    "#,
    );

    assert_snapshot!(print_js(&document));
}

fn parse(str: &str) -> OperationDocument {
    let doc = parse_operation_document(str).unwrap();
    let (document, _) = resolve_operation_extensions(doc).unwrap();
    document
}

fn print_js(document: &OperationDocument) -> String {
    let mut buffer = String::new();
    let mut writer = JustWriter::new(&mut buffer);
    let options = OperationJSPrinterOptions::default();
    print_js_for_operation_document(options, document, &mut writer);
    buffer
}
