use std::path::Path;

use insta::assert_snapshot;
use nitrogql_ast::OperationDocument;
use nitrogql_parser::parse_operation_document;
use nitrogql_printer::GraphQLPrinter;
use sourcemap_writer::JustWriter;

use crate::{
    resolve_operation_extensions, resolve_operation_imports, OperationExtension, OperationResolver,
};

struct TestOperationResolver;
impl OperationResolver<'static> for TestOperationResolver {
    fn resolve(
        &self,
        path: &Path,
    ) -> Option<(&OperationDocument<'static>, &OperationExtension<'static>)> {
        match path.to_str().unwrap() {
            "/path/to/frag1.graphql" => Some(static_parse(
                r#"
                        fragment Frag1 on Foo {
                            bar
                        }
                        "#,
            )),
            _ => None,
        }
    }
}

fn static_parse(
    code: &'static str,
) -> (
    &'static OperationDocument<'static>,
    &'static OperationExtension<'static>,
) {
    let doc = parse_operation_document(code).unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let (doc, extensions) = (Box::leak(Box::new(doc)), Box::leak(Box::new(extensions)));
    (doc, extensions)
}

#[test]
fn no_import() {
    let doc = parse_operation_document(
        r#"
        query Foo {
            foo
        }
        "#,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
    let resolved = resolve_operation_imports(doc, &TestOperationResolver).unwrap();
    assert_snapshot!(print_document(&resolved));
}

#[test]
fn specific_import() {
    let doc = parse_operation_document(
        r#"
        #import Frag1 from "./frag1.graphql"
        query Foo {
            foo {
                ...Frag1
            }
        }
        "#,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
    let resolved = resolve_operation_imports(doc, &TestOperationResolver).unwrap();
    assert_snapshot!(print_document(&resolved));
}

fn print_document(document: &OperationDocument) -> String {
    let mut buffer = String::new();
    let mut printer = JustWriter::new(&mut buffer);
    document.print_graphql(&mut printer);
    buffer
}
