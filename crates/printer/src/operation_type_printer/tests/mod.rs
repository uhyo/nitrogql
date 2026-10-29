use std::path::Path;

use insta::assert_snapshot;

use graphql_builtins::generate_builtins;
use nitrogql_ast::OperationDocumentExt;
use nitrogql_ast::{OperationDocument, TypeSystemDocument};
use nitrogql_parser::{parse_operation_document, parse_type_system_document};
use nitrogql_semantics::{
    OperationExtension, OperationResolver, ast_to_type_system, resolve_operation_extensions,
};
use nitrogql_semantics::{resolve_operation_imports, resolve_schema_extensions};
use sourcemap_writer::JustWriter;

use crate::OperationTypePrinterOptions;
use crate::operation_base_printer::options::OperationBasePrinterOptions;
use crate::print_types_for_operation_document;

mod fragment_refs;

const TYPE_SYSTEM: &str = "
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

            type Tweet implements HasID {
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
                posts: [Post!]
                nullablePosts: [Post]!
            }
";

fn type_system() -> TypeSystemDocument<'static> {
    type_system_from(TYPE_SYSTEM)
}

fn type_system_from(source: &'static str) -> TypeSystemDocument<'static> {
    let mut doc = parse_type_system_document(source, 0).unwrap();
    doc.extend(generate_builtins());

    resolve_schema_extensions(doc).unwrap()
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
        0,
    )
    .unwrap();
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn nested_list_and_non_null() {
    let doc = parse_operation_document(
        "
        query {
            posts { id title }
            nullablePosts { id body }
        }
        ",
        0,
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
        0,
    )
    .unwrap();
    let (doc, _) = resolve_operation_extensions(doc).unwrap();
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
fn descriptions() {
    let doc = parse_operation_document(
        r#"
        """
        Query to get the current user.
        """
        query testQuery(
            "ID of the user."
            $foo: Int!
            "Name of the user."
            $bar: String
        ) {
            me(foo: $foo, bar: $bar) {
                id name type age
                ...F
            }
        }
        "Fragment to get the ID."
        fragment F on HasID {
            id
        }
        "#,
        0,
    )
    .unwrap();
    let printed = print_document_default(&doc);
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
        0,
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
        0,
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
        0,
    )
    .unwrap();
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn typename_field() {
    let doc = parse_operation_document(
        "
        query {
            me {
                __typename
                id name type age
            }
        }
        ",
        0,
    )
    .unwrap();
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn typename_field_on_union() {
    let doc = parse_operation_document(
        "
        query {
            me {
                posts {
                    __typename
                    id
                    ... on Post {
                        title
                    }
                }
            }
        }
        ",
        0,
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
        0,
    )
    .unwrap();
    let printed = print_document_default(&doc);
    assert_snapshot!(printed);
}

#[test]
fn variable_disallow_undefined() {
    let doc = parse_operation_document(
        "
        query testQuery($foo: Int!, $bar: String) {
            me(foo: $foo, bar: $bar) {
                id name type age
            }
        }
        ",
        0,
    )
    .unwrap();
    let (doc, _) = resolve_operation_extensions(doc).unwrap();
    let printed = print_document(
        &doc,
        OperationTypePrinterOptions {
            base_options: Default::default(),
            allow_undefined_as_optional_input: false,
            ..Default::default()
        },
    );
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
        0,
    )
    .unwrap();
    let (doc, _) = resolve_operation_extensions(doc).unwrap();
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let schema = type_system();
    let schema = ast_to_type_system(&schema);
    let options = OperationTypePrinterOptions {
        print_values: true,
        ..Default::default()
    };
    print_types_for_operation_document(options, &schema, &doc, &mut writer).unwrap();
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
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
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }
}

mod fragment_merging {
    use super::*;

    #[test]
    fn fragment_merging() {
        let doc = parse_operation_document(
            "
        query {
            me {
                ...F
                ...P
            }
        }
        fragment F on User {
            id
        }
        fragment P on User {
            name
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn fragment_nested_merging() {
        let doc = parse_operation_document(
            "
        query {
            ...F
            ...P
        }
        fragment F on Query {
            me {
                id
            }
        }
        fragment P on Query {
            me {
                name
            }
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn fragment_nested_merging_list() {
        let doc = parse_operation_document(
            "
        query {
            ...F
            ...P
        }
        fragment F on Query {
            posts {
                id
                tags
            }
        }
        fragment P on Query {
            posts {
                id
                title
            }
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn conditional_fragments() {
        let doc = parse_operation_document(
            "
        query {
            ...F
            ...P
        }
        fragment F on Query {
            me {
                posts {
                    id
                    ... on Post {
                        title
                    }
                }
            }
        }
        fragment P on Query {
            me {
                posts {
                    id
                    ... on Tweet {
                        body
                    }
                }
            }
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn conditional_include() {
        let doc = parse_operation_document(
            "
        query($cond: Boolean!) {
            ... on Query @include(if: $cond) {
                posts {
                    id
                    title
                }
            }
            ... on Query @skip(if: $cond) {
                posts {
                    tags
                    body
                }
            }
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn variables_in_nested_merging() {
        let doc = parse_operation_document(
            "
        query($x: Boolean!, $y: Boolean!) {
            me {
                name @include(if: $x)
            }
            ...F
        }
        fragment F on Query {
            me {
                age @include(if: $y)
            }
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }

    #[test]
    fn same_variable_in_nested_merging() {
        let doc = parse_operation_document(
            "
        query($x: Boolean!) {
            me {
                name @include(if: $x)
            }
            ...F
        }
        fragment F on Query {
            me {
                age @include(if: $x)
            }
        }
        ",
            0,
        )
        .unwrap();
        let printed = print_document_default(&doc);
        assert_snapshot!(printed);
    }
}

mod import_fragments {
    use super::*;

    #[test]
    fn import_fragment() {
        let doc = parse_operation_document(
            "
            #import UserProfile from \"./user-profile.graphql\"
            query myQuery {
                me {
                    id 
                    ...UserProfile
                }
            }
            ",
            0,
        )
        .unwrap();
        let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
        let doc = (Path::new("/path/to/main.graphql"), &doc, &extensions);
        let resolved = resolve_operation_imports(doc, &TestOperationResolver).unwrap();
        let printed = print_document(&resolved, OperationTypePrinterOptions::default());
        assert_snapshot!(printed);
    }
}

struct TestOperationResolver;
impl<'src> OperationResolver<'src> for TestOperationResolver {
    fn resolve(
        &self,
        path: &Path,
    ) -> Option<(
        &OperationDocument<'src>,
        &nitrogql_semantics::OperationExtension<'src>,
    )> {
        match path.to_str().unwrap() {
            "/path/to/user-profile.graphql" => Some(static_parse(
                r#"
fragment UserProfile on User {
    name
    age
}
"#,
                1,
            )),
            _ => None,
        }
    }
}

/// Documents that are not valid against the schema make the printer return an error.
mod type_system_errors {
    use super::*;

    /// Prints given (unchecked) document and returns the error as `line:column: message`.
    fn print_error(source: &str) -> String {
        let doc = parse_operation_document(source, 0).unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        let schema = type_system();
        let schema = ast_to_type_system(&schema);
        let err = print_types_for_operation_document(
            OperationTypePrinterOptions::default(),
            &schema,
            &doc,
            &mut writer,
        )
        .unwrap_err();
        format!(
            "{}:{}: {}",
            err.position.line + 1,
            err.position.column + 1,
            err.message
        )
    }

    #[test]
    fn unknown_field() {
        assert_eq!(
            print_error("query {\n  me { foo }\n}"),
            "2:8: Field 'foo' is not defined on type 'User'"
        );
    }

    #[test]
    fn selection_on_scalar() {
        assert_eq!(
            print_error("query {\n  me { id { foo } }\n}"),
            "2:11: Type 'ID' does not have fields"
        );
    }

    #[test]
    fn unknown_fragment() {
        assert_eq!(
            print_error("query {\n  me { ...F }\n}"),
            "2:11: Fragment 'F' is not defined"
        );
    }

    #[test]
    fn unknown_type_condition() {
        assert_eq!(
            print_error("query {\n  me { ... on Foo { id } }\n}"),
            "2:15: Type 'Foo' is not defined"
        );
    }

    #[test]
    fn missing_root_type() {
        assert_eq!(
            print_error("mutation {\n  foo\n}"),
            "1:10: Type 'Mutation' is not defined"
        );
    }

    #[test]
    fn missing_if_argument() {
        assert_eq!(
            print_error("query {\n  me { id @skip }\n}"),
            "2:11: Directive '@skip' needs an 'if' argument"
        );
    }

    #[test]
    fn field_shape_mismatch() {
        assert_eq!(
            print_error("query {\n  me { a: name a: posts { id } }\n}"),
            "2:8: Cannot merge selections of field 'a' that have different shapes"
        );
    }

    #[test]
    fn nested_field_shape_mismatch() {
        assert_eq!(
            print_error("query {\n  me { a: name }\n  me { a: posts { id } }\n}"),
            "2:3: Cannot merge selections of field 'a' that have different shapes"
        );
    }
}

mod per_definition_fallback {
    use nitrogql_ast::{base::Pos, operation::ExecutableDefinition};

    use super::*;
    use crate::{PerDefinitionTypeSource, print_types_with_source};

    /// Prints given document with definitions of given names falling back to `any`.
    fn print(source: &str, fallback_names: &[&str]) -> (String, Vec<String>) {
        let doc = parse_operation_document(source, 0).unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let fallback_positions = doc
            .definitions
            .iter()
            .filter_map(|def| match def {
                ExecutableDefinition::OperationDefinition(def) => def
                    .name
                    .filter(|name| fallback_names.contains(&name.name))
                    .map(|_| def.position),
                ExecutableDefinition::FragmentDefinition(def) => fallback_names
                    .contains(&def.name.name)
                    .then_some(def.position),
            })
            .collect::<Vec<Pos>>();
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        let schema = type_system();
        let schema = ast_to_type_system(&schema);
        let options = OperationTypePrinterOptions::default();
        let types = PerDefinitionTypeSource::new(&options, &schema, &doc, |pos| {
            fallback_positions.contains(pos)
        });
        let Ok(types) = print_types_with_source(&options, types, &doc, &mut writer);
        let errors = types
            .into_errors()
            .into_iter()
            .map(|err| {
                format!(
                    "{}:{}: {}",
                    err.position.line + 1,
                    err.position.column + 1,
                    err.message
                )
            })
            .collect();
        (result, errors)
    }

    #[test]
    fn fallback_for_some_definitions() {
        let (printed, errors) = print(
            "
            query A { me { id } }
            query B { me { unknownField } }
            fragment F on User { name }
            fragment G on UnknownType { foo }
            ",
            &["B", "G"],
        );
        assert_snapshot!(printed);
        assert!(errors.is_empty());
    }

    #[test]
    fn fallback_for_all_definitions() {
        // Schema types are not imported if no definition uses them.
        let (printed, errors) = print(
            "
            query B { me { unknownField } }
            fragment G on UnknownType { foo }
            ",
            &["B", "G"],
        );
        assert_snapshot!(printed);
        assert!(errors.is_empty());
    }

    #[test]
    fn fallback_on_printer_error() {
        // Definitions that cannot be printed fall back to `any` without affecting others.
        let (printed, errors) = print(
            "query A { me { id } }\nquery B { a: __typename a: me { id } }\n",
            &[],
        );
        assert_snapshot!(printed);
        assert_eq!(
            errors,
            vec!["2:11: Cannot merge selections of field 'a' that have different shapes"]
        );
    }
}

/// All types are `any` with `AnyTypeSource`.
mod any_type_source {
    use super::*;
    use crate::{AnyTypeSource, print_types_with_source};

    fn print(source: &str, options: OperationTypePrinterOptions) -> String {
        let doc = parse_operation_document(source, 0).unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let mut result = String::new();
        let mut writer = JustWriter::new(&mut result);
        let Ok(AnyTypeSource) = print_types_with_source(&options, AnyTypeSource, &doc, &mut writer);
        result
    }

    #[test]
    fn fallback_types() {
        // Fields do not need to exist in any schema.
        let printed = print(
            "
            query GetMe($id: ID!) {
                me(id: $id) { unknownField }
            }
            fragment F on UnknownType {
                foo
            }
            ",
            OperationTypePrinterOptions::default(),
        );
        assert_snapshot!(printed);
    }

    #[test]
    fn fallback_types_with_descriptions() {
        // Descriptions are printed in the same way as the schema-based printer.
        let printed = print(
            r#"
            "Get the current user"
            query GetMe { me { unknownField } }
            "User data"
            fragment F on UnknownType { foo }
            "#,
            OperationTypePrinterOptions::default(),
        );
        assert_snapshot!(printed);
    }

    #[test]
    fn fallback_types_named_exports() {
        let mut options = OperationTypePrinterOptions::default();
        options.base_options.default_export_for_operation = false;
        options.base_options.named_export_for_operation = true;
        options.base_options.export_result_type = true;
        options.base_options.export_input_type = true;
        let printed = print(
            "
            query A { a }
            mutation B { b }
            ",
            options,
        );
        assert_snapshot!(printed);
    }
}

fn static_parse(
    code: &'static str,
    file_id: usize,
) -> (
    &'static OperationDocument<'static>,
    &'static OperationExtension<'static>,
) {
    let doc = parse_operation_document(code, file_id).unwrap();
    let (doc, extensions) = resolve_operation_extensions(doc).unwrap();
    let (doc, extensions) = (Box::leak(Box::new(doc)), Box::leak(Box::new(extensions)));
    (doc, extensions)
}

fn print_document_default(document: &OperationDocumentExt) -> String {
    let (document, extensions) = resolve_operation_extensions(document.clone()).unwrap();
    let document = resolve_operation_imports(
        (Path::new("/path/to/main.graphql"), &document, &extensions),
        &TestOperationResolver,
    )
    .unwrap();
    print_document(&document, OperationTypePrinterOptions::default())
}

fn print_document(document: &OperationDocument, options: OperationTypePrinterOptions) -> String {
    let mut result = String::new();
    let mut writer = JustWriter::new(&mut result);
    let schema = type_system();
    let schema = ast_to_type_system(&schema);
    print_types_for_operation_document(options, &schema, document, &mut writer).unwrap();
    result
}

/// Mappings of named definitions point to their names.
mod mappings {
    use super::*;
    use sourcemap_writer::SpanWriter;

    #[test]
    fn mappings_point_to_names() {
        let source = "\"Description\"\nfragment UserName on User { name }\nquery GetMe { me { ...UserName } }\n";
        let doc = parse_operation_document(source, 0).unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let schema = type_system();
        let schema = ast_to_type_system(&schema);
        let mut writer = SpanWriter::new();
        print_types_for_operation_document(
            OperationTypePrinterOptions::default(),
            &schema,
            &doc,
            &mut writer,
        )
        .unwrap();
        let (_, spans) = writer.into_parts();
        assert!(!spans.is_empty());
        for span in spans {
            assert_eq!(
                &source[span.original_pos.span()],
                span.original_name,
                "{span:?}"
            );
        }
    }
}
