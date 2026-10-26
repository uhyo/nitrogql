use std::path::Path;

use insta::{assert_debug_snapshot, assert_snapshot};
use nitrogql_ast::OperationDocument;
use nitrogql_parser::parse_operation_document;
use nitrogql_printer::GraphQLPrinter;
use sourcemap_writer::JustWriter;

use crate::{
    OperationExtension, OperationResolver, directly_imported_fragments,
    resolve_operation_extensions, resolve_operation_imports,
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
fragment Frag3_1 on Foo {
    bar
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
    let doc = parse_operation_document(code, 0).unwrap();
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
        0,
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
        0,
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
        0,
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
        0,
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
        0,
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
        0,
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
        0,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
    let resolved = resolve_operation_imports(doc, &TestOperationResolver).unwrap();
    assert_snapshot!(print_document(&resolved));
}

#[test]
fn import_from_visited_file() {
    // frag3.graphql is visited through frag2.graphql first, but Frag3_1 is imported only directly
    let doc = parse_operation_document(
        r#"
        #import Frag2 from "./frag2.graphql"
        #import Frag3_1 from "./frag3.graphql"
        query Foo {
            foo {
                ...Frag2
                ...Frag3_1
            }
        }
        "#,
        0,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
    let resolved = resolve_operation_imports(doc, &TestOperationResolver).unwrap();
    assert_snapshot!(print_document(&resolved));
}

#[test]
fn import_cycle_to_root() {
    // the document itself is imported back by frag1.graphql
    let doc = parse_operation_document(
        r#"
        #import Frag1 from "./frag1.graphql"
        fragment Frag2 on Foo { baz ...Frag1 }
        "#,
        0,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/rec/frag2.graphql"), &doc, &extensions);
    let resolved = resolve_operation_imports(doc, &TestOperationResolver).unwrap();
    assert_snapshot!(print_document(&resolved));
}

#[test]
fn error_import_fragment_nonexistent_in_visited_file() {
    let doc = parse_operation_document(
        r#"
        #import Frag2 from "./frag2.graphql"
        #import Frag999 from "./frag3.graphql"
        query Foo {
            foo {
                ...Frag2
            }
        }
        "#,
        0,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
    let err = resolve_operation_imports(doc, &TestOperationResolver).unwrap_err();
    assert_debug_snapshot!(err);
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
        0,
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
        0,
    )
    .unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
    let err = resolve_operation_imports(doc, &TestOperationResolver).unwrap_err();
    assert_debug_snapshot!(err);
}

#[test]
fn directly_imported_fragments_excludes_transitive_imports() {
    let doc = parse_operation_document(
        r#"
        #import Frag2 from "./frag2.graphql"
        #import * from "./frag1.graphql"
        query Foo {
            foo {
                ...Frag2
            }
        }
        "#,
        0,
    )
    .unwrap();
    let (_, extensions) = resolve_operation_extensions(doc).unwrap();
    let imported = directly_imported_fragments(
        (Path::new("/path/to/main.graphql"), &extensions),
        &TestOperationResolver,
    );
    let mut imported = imported.into_iter().collect::<Vec<_>>();
    imported.sort();
    // Frag3 is imported by frag2.graphql, not by this document
    assert_eq!(imported, ["Frag1", "Frag1_1", "Frag2"]);
}

fn print_document(document: &OperationDocument) -> String {
    let mut buffer = String::new();
    let mut printer = JustWriter::new(&mut buffer);
    document.print_graphql(&mut printer);
    buffer
}
