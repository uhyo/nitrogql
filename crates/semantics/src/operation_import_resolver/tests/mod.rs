use std::path::Path;

use insta::{assert_debug_snapshot, assert_snapshot};
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
fragment Frag1_1 on Foo {
    baz
}
"#,
            )),
            "/path/to/frag2.graphql" => Some(static_parse(
                r#"
#import Frag3 from "./frag3.graphql"
fragment Frag2 on Foo {
    bar
    ...Frag3
}
"#,
            )),
            "/path/to/frag3.graphql" => Some(static_parse(
                r#"
fragment Frag3 on Foo {
    baz
}
"#,
            )),
            "/path/to/rec/frag1.graphql" => Some(static_parse(
                r#"
#import Frag2 from "frag2.graphql"
fragment Frag1 on Foo { bar }
"#,
            )),
            "/path/to/rec/frag2.graphql" => Some(static_parse(
                r#"
#import Frag1 from "frag1.graphql"
fragment Frag2 on Foo { baz }
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

#[test]
fn transitive_import() {
    let doc = parse_operation_document(
        r#"
        #import Frag2 from "./frag2.graphql"
        query Foo {
            foo {
                ...Frag2
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

#[test]
fn multiple_imports() {
    let doc = parse_operation_document(
        r#"
        #import Frag1 from "./frag1.graphql"
        #import Frag3 from "./frag3.graphql"
        query Foo {
            foo {
                ...Frag1
                ...Frag3
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

#[test]
fn import_wildcard() {
    let doc = parse_operation_document(
        r#"
        #import * from "./frag1.graphql"
        query Foo {
            foo {
                ...Frag1
                ...Frag1_1
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

#[test]
fn recursive_import() {
    let doc = parse_operation_document(
        r#"
        #import Frag1 from "./rec/frag1.graphql"
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

#[test]
fn import_twice() {
    let doc = parse_operation_document(
        r#"
        #import Frag1 from "./frag1.graphql"
        #import Frag1_1 from "./frag1.graphql"
        query Foo {
            foo {
                ...Frag1
                ...Frag1_1
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

#[test]
fn error_import_nonexistent() {
    let doc = parse_operation_document(
        r#"
        #import Frag1 from "./nonexistent.graphql"
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
    let err = resolve_operation_imports(doc, &TestOperationResolver).unwrap_err();
    assert_debug_snapshot!(err);
}

#[test]
fn error_import_fragment_nonexistent() {
    let doc = parse_operation_document(
        r#"
        #import Frag999 from "./frag1.graphql"
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
    let err = resolve_operation_imports(doc, &TestOperationResolver).unwrap_err();
    assert_debug_snapshot!(err);
}

fn print_document(document: &OperationDocument) -> String {
    let mut buffer = String::new();
    let mut printer = JustWriter::new(&mut buffer);
    document.print_graphql(&mut printer);
    buffer
}
