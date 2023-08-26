use insta::assert_snapshot;

use graphql_builtins::generate_builtins;
use nitrogql_ast::{OperationDocument, TypeSystemDocument};
use nitrogql_parser::{parse_operation_document, parse_type_system_document};
use nitrogql_semantics::ast_to_type_system;
use nitrogql_semantics::resolve_extensions;
use sourcemap_writer::JustWriter;

use crate::operation_base_printer::options::OperationBasePrinterOptions;
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
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn export_input_and_result_type() {
    let doc = parse_operation_document(
        "
        query sampleQuery {
            me {
                id name type age
            }
        }
        ",
    )
    .unwrap();
    let printed = print_document(
        &doc,
        OperationTypePrinterOptions {
            base_options: OperationBasePrinterOptions {
                export_input_type: true,
                export_result_type: true,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    assert_snapshot!(printed);
}

#[test]
fn fragment_spread() {
    let doc = parse_operation_document(
        "
        query test{
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
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn fragment_spread_cond() {
    let doc = parse_operation_document(
        "
        query fooBar123 {
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
    let printed = print_document_default(&doc);
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
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn query_variables() {
    let doc = parse_operation_document(
        "
        query testQuery($foo: Int!, $bar: String) {
            me(foo: $foo, bar: $bar) {
                id name type age
            }
        }
        ",
    )
    .unwrap();
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn print_values() {
    let doc = parse_operation_document(
        "
        query {
            me {
                id name
            }
        }
        ",
    )
    .unwrap();
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let schema = type_system();
    let schema = ast_to_type_system(&schema);
    let options = OperationTypePrinterOptions {
        print_values: true,
        ..Default::default()
    };
    print_types_for_operation_document(options, &schema, &doc, &mut writer);
    assert_snapshot!(result);
}

mod skip_include {
    use super::*;

    #[test]
    fn skipped_field_is_not_included() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    age @skip(if: true)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn not_skipped_field_is_included() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    age @skip(if: false)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn included_field_is_included() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    age @include(if: true)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn not_included_field_is_not_included() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    age @include(if: false)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_variable() {
        let doc = parse_operation_document(
            "
            query($skip: Boolean!) {
                me {
                    id name type
                    age @skip(if: $skip)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn include_variable() {
        let doc = parse_operation_document(
            "
            query($include: Boolean!) {
                me {
                    id name type
                    age @include(if: $include)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_and_include() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id @skip(if: false) @include(if: false)
                    name @skip(if: true) @include(if: false)
                    type @skip(if: false) @include(if: true)
                    age @skip(if: true) @include(if: true)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn emit_only_possible_cases() {
        let doc = parse_operation_document(
            "
            query($flag: Boolean!) {
                me {
                    id @skip(if: $flag)
                    name @include(if: $flag)
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_fragment_spread() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    ...F @skip(if: true)
                }
            }
            fragment F on User {
                age
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn include_fragment_spread() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    ...F @include(if: true)
                }
            }
            fragment F on User {
                age
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_inline_fragment() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    ... @skip(if: true) {
                        age
                    }
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn include_inline_fragment() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    ... @include(if: true) {
                        age
                    }
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_and_include_fragment_spread() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    ...F @skip(if: true) @include(if: true)
                }
            }
            fragment F on User {
                age
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_and_include_inline_fragment() {
        let doc = parse_operation_document(
            "
            query {
                me {
                    id name type
                    ... @skip(if: true) @include(if: true) {
                        age
                    }
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_fragment_spread_variable() {
        let doc = parse_operation_document(
            "
            query($skip: Boolean!) {
                me {
                    id name type
                    ...F @skip(if: $skip)
                }
            }
            fragment F on User {
                age
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn include_fragment_spread_variable() {
        let doc = parse_operation_document(
            "
            query($include: Boolean!) {
                me {
                    id name type
                    ...F @include(if: $include)
                }
            }
            fragment F on User {
                age
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_inline_fragment_variable() {
        let doc = parse_operation_document(
            "
            query($skip: Boolean!) {
                me {
                    id name type
                    ... @skip(if: $skip) {
                        age
                    }
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn include_inline_fragment_variable() {
        let doc = parse_operation_document(
            "
            query($include: Boolean!) {
                me {
                    id name type
                    ... @include(if: $include) {
                        age
                    }
                }
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn skip_and_include_fragment_spread_variable() {
        let doc = parse_operation_document(
            "
            query($flag: Boolean!) {
                me {
                    id name
                    ...F @skip(if: $flag)
                    ... @include(if: $flag) {
                        age
                    }
                }
            }
            fragment F on User {
                type
            }
            ",
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }
}

fn print_document_default(document: &OperationDocument) -> String {
    print_document(document, OperationTypePrinterOptions::default())
}

fn print_document(document: &OperationDocument, options: OperationTypePrinterOptions) -> String {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let schema = type_system();
    let schema = ast_to_type_system(&schema);
    print_types_for_operation_document(options, &schema, document, &mut writer);
    result
}
