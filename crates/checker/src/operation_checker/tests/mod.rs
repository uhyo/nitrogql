use std::borrow::Cow;

use graphql_builtins::generate_builtins;
use graphql_type_system::Schema;
use nitrogql_ast::{OperationDocumentExt, TypeSystemDocument, base::Pos};
use nitrogql_parser::parse_type_system_document;
use nitrogql_semantics::{resolve_operation_extensions, resolve_schema_extensions};

use crate::{CheckError, OperationCheckContext, check_operation_document};

mod operations {
    use std::borrow::Cow;

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::ast_to_type_system;

    use nitrogql_ast::base::Pos;
    use nitrogql_parser::parse_operation_document;

    use super::{parse_to_type_system_document, test_check};

    fn type_system() -> Schema<Cow<'static, str>, Pos> {
        let doc = parse_to_type_system_document(
            "
            schema {
                subscription: S
            }

            type S {
                foo: [Foo!]!
                bar: [String!]!
            }
            type Foo {
                hoge: Int!
                piyo: Int!
            }
        ",
        );
        ast_to_type_system(&doc)
    }

    #[test]
    fn subscription_root_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            subscription a { foo { hoge piyo } }
            subscription b { foo { hoge } bar }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn subscription_root_field_recursing() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            subscription { ...F }
            fragment F on S {
                ...F
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }
}

mod operation_directives {
    use std::borrow::Cow;

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::ast_to_type_system;

    use nitrogql_ast::base::Pos;
    use nitrogql_parser::parse_operation_document;

    use super::{parse_to_type_system_document, test_check};

    fn type_system() -> Schema<Cow<'static, str>, Pos> {
        let doc = parse_to_type_system_document(
            "
            directive @dir_bool_nonnull(bool: Boolean!) repeatable on QUERY | FIELD
            directive @dir_bool(bool: Boolean) repeatable on MUTATION | FIELD

            type Query {
                foo: Int!
            }
            type Mutation {
                bar: Int!
            }

            directive @dir_input(input: MyInput) on QUERY | FIELD
            input MyInput {
                bool: Boolean
                int: Int!
            }
        ",
        );
        ast_to_type_system(&doc)
    }

    #[test]
    fn unknown_directive() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query @unknown_dir {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }
    #[test]
    fn wrong_location() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query @dir_bool {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_argument_type() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query @dir_bool_nonnull(bool: 3) {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn unknown_argument() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query @dir_bool_nonnull(bool: false, another: true) {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn no_argument() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query q @dir_bool_nonnull {
                foo
            }
            mutation m
              @dir_bool
              @dir_bool(bool: null)
              @dir_bool(bool: true) {
                bar
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn variable_check() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query q($b: Boolean, $b2: Boolean!, $i: Int!)
                @dir_bool_nonnull(bool: $b)
                @dir_bool_nonnull(bool: $b2)
                @dir_bool_nonnull(bool: $i)
                 {
                foo
                @dir_bool(bool: $b)
                @dir_bool(bool: $b2)
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn unknown_variable() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query q1 @dir_bool_nonnull(bool: $b) {
                foo
                @dir_bool(bool: $b)
            }
            query q2($b: Boolean!) @dir_bool_nonnull(bool: $b2) {
                foo @dir_bool(bool: $b2)
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn input_missing_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            @dir_input(input: { bool: true })
            {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn input_extra_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            @dir_input(input: { bool: true, int: 3 str: \"foobar\" })
            {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn input_field_type_mismatch() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            @dir_input(input: { bool: true, int: \"foobar\" })
            {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn input_field_null_literal() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            @dir_input(input: { bool: null, int: 3 })
            {
                foo
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }
}

mod selection_set {
    use std::borrow::Cow;

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::ast_to_type_system;

    use nitrogql_ast::base::Pos;
    use nitrogql_parser::parse_operation_document;

    use super::{parse_to_type_system_document, test_check};

    fn type_system() -> Schema<Cow<'static, str>, Pos> {
        let doc = parse_to_type_system_document(
            "
            type Query {
                foo: Int!
                user: User
                users(name: String): [User!]!
            }
            type User {
                id: ID!
                name: String!
                age: Int
                pet: Animal
                relations: [UserOrPost!]!
            }
            interface Animal {
                id: ID!
                name: String!
            }
            type Post {
                id: ID!
                title: String!
            }
            union UserOrPost = User | Post
        ",
        );
        ast_to_type_system(&doc)
    }

    #[test]
    fn unknown_selected_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            {
                user { id }
                user2
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn duplicated_selected_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query q1 {
                user { id }
                user { id name }
            }
            query q2 {
                user { id }
                alias: user { id name } # ← this is ok
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn duplicated_alias() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            {
                user { id }
                user: foo 
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn invalid_nested_selection() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                foo { value }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn check_field_arguments() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                users(name: \"uhyo\") { id name age }
                user(arg: 123) { id }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn check_field_arguments_variable() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query($str: String!, $num: Int!, $maybeNum: Int) {
                users1: users(name: $str) { id }
                users2: users(name: $num) { id }
                users2: users(name: $maybeNum) { id }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn must_specify_selection_set_for_non_leaf_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                foo
                user
                users(name: \"uhyo\")
                user2: user {
                    id
                    name
                    age
                }
                users2: users {
                    id
                    name
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn supports_typename_metafield() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user {
                    __typename
                    pet {
                        __typename
                    }
                    relations {
                        __typename
                    }
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }
}

mod fragments {
    use std::borrow::Cow;

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::{ast_to_type_system, resolve_operation_extensions};

    use crate::{OperationCheckContext, operation_checker::check_operation_document};
    use nitrogql_ast::base::Pos;
    use nitrogql_parser::parse_operation_document;

    use super::{parse_to_type_system_document, test_check};

    fn type_system() -> Schema<Cow<'static, str>, Pos> {
        let doc = parse_to_type_system_document(
            "
            directive @dir_bool(bool: Boolean!) on FIELD
            scalar CustomScalar
            
            type Query {
                foo: Int!
                user: User
                users(name: String): [User!]!
                hasTitle: HasTitle!
                postOrTag: PostOrTag
                userOrSchedule: UserOrSchedule
            }
            type User {
                id: ID!
                name: String!
                age: Int
                userKind: UserKind
                followers: [User!]!
            }
            enum UserKind { NormalUser PremiumUser }
            input MyInput {
                arg: String! = \"\"
            }

            type Post implements HasTitle {
                id: ID!
                title: String!
                body: String!
            }
            interface HasTitle {
                title: String!
            }
            interface HasLabel {
                label: String!
            }

            type Tag implements HasLabel {
                id: ID!
                label: String!
            }
            type Schedule {
                id: ID!
            }
            union UserOrTag = User | Tag
            union PostOrTag = Post | Tag
            union UserOrSchedule = User | Schedule
        ",
        );
        ast_to_type_system(&doc)
    }

    #[test]
    fn unknown_fragment_target() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user { id } }
            fragment A on Nothing {
                id
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn imported_fragment() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user { ...Imported ...Unknown } }
        ",
            0,
        )
        .unwrap();
        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let is_imported = |name: &str| name == "Imported";
        let context = OperationCheckContext::new(&schema).with_imported_fragments(&is_imported);
        let errors = check_operation_document(&doc, &context)
            .into_iter()
            .map(|err| err.message.to_string())
            .collect::<Vec<_>>();
        assert_eq!(errors, vec!["Fragment 'Unknown' is not defined"]);
    }

    #[test]
    fn fragment_not_spread() {
        // Fragments that are not spread from operations are checked on their own.
        // Variables are not known, so they are not checked.
        let schema = type_system();
        let doc = parse_operation_document(
            "
            fragment A on Query { foo nam users(name: $name) { id } }
            fragment B on Query { ...A }
        ",
            0,
        )
        .unwrap();
        let errors = test_check(schema, doc)
            .into_iter()
            .map(|err| err.message.to_string())
            .collect::<Vec<_>>();
        assert_eq!(errors, vec!["Field 'nam' is not found on type 'Query'"]);
    }

    #[test]
    fn invalid_fragment_target() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user { id }}
            fragment OnScalar on CustomScalar {
                id
            }
            fragment OnEnum on UserKind {
                id
            }
            fragment OnInput on MyInput {
                arg
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_fragment_target_obj() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user {
                ...F
            }}
            fragment F on Post {
                id
                title
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_fragment_target_intf() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user {
                ...F
            }}
            fragment F on HasTitle {
                title
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_fragment_target_union() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user {
                ...F
            }}
            fragment F on PostOrTag {
                ... on Post {
                    id
                }
                ... on Tag {
                    id
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_fragment_target_intf_intf() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { hasTitle {
                ...F
                ...G
                ...H
            }}
            fragment F on HasLabel {
                label
            }
            fragment G on HasTitle {
                title
            }
            fragment H on Post {
                title
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_fragment_target_intf_union() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { hasTitle {
                ...F
                ...G
            }}
            fragment F on PostOrTag {
                ... on Post {
                    title
                }
                ... on Tag {
                    label
                }
            }
            fragment G on UserOrPost {
                ... on Post {
                    title
                }
                ... on User {
                    name
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_fragment_target_union_union() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { userOrSchedule {
                ...F
                ...G
            }}
            fragment F on PostOrTag {
                ... on Post {
                    title
                }
                ... on Tag {
                    label
                }
            }
            fragment G on UserOrTag {
                ... on Tag {
                    label
                }
                ... on User {
                    name
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn fragment_variables() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query($b1: Boolean!) { user {
                ...F
            }}
            fragment F on User {
                id @dir_bool(bool: $b1)
                name @dir_bool(bool: $b2)
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn inline_fragment_without_condition() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query($b1: Boolean!) { user {
                id
                ... {
                    name
                    age
                }
                ... {
                    something
                }
            }}
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_inline_fragment_target_obj() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query { user {
                ... on Post {
                    id
                    title
                }
            }}
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_inline_fragment_target_intf() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user {
                    ... on HasTitle {
                        title
                    }
                }
                hasTitle {
                    ... on HasTitle {
                        title
                    }
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn wrong_inline_fragment_target_union() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user {
                    ... on PostOrTag {
                        ... on Post {
                            id
                        }
                        ... on Tag {
                            id
                        }
                    }
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn fragment_spread_recursion() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user {
                    ...F
                }
            }
            fragment F on User {
                id
                ...F
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn fragment_spread_indirect_recursion() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user {
                    ...F
                }
            }
            fragment F on User {
                id
                ...G
            }
            fragment G on User {
                age
                ...F
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn fragment_spread_nested_recursion() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user {
                    ...F
                }
            }
            fragment F on User {
                followers {
                    id
                    ...F
                }
            }
        ",
            0,
        )
        .unwrap();

        let (doc, _) = resolve_operation_extensions(doc).unwrap();
        let context = OperationCheckContext::new(&schema);
        assert_debug_snapshot!(check_operation_document(&doc, &context));
    }
}

mod imports {
    use std::borrow::Cow;

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::ast_to_type_system;

    use crate::operation_checker::tests::test_check;
    use nitrogql_ast::base::Pos;
    use nitrogql_parser::parse_operation_document;

    use super::parse_to_type_system_document;

    fn type_system() -> Schema<Cow<'static, str>, Pos> {
        let doc = parse_to_type_system_document(
            "
            directive @dir_bool(bool: Boolean!) on FIELD
            scalar CustomScalar
            
            type Query {
                foo: Int!
                user: User
                users(name: String): [User!]!
                hasTitle: HasTitle!
                postOrTag: PostOrTag
                userOrSchedule: UserOrSchedule
            }
            type User {
                id: ID!
                name: String!
                age: Int
                userKind: UserKind
                followers: [User!]!
            }
            enum UserKind { NormalUser PremiumUser }
            input MyInput {
                arg: String! = \"\"
            }

            type Post implements HasTitle {
                id: ID!
                title: String!
                body: String!
            }
            interface HasTitle {
                title: String!
            }
            interface HasLabel {
                label: String!
            }

            type Tag implements HasLabel {
                id: ID!
                label: String!
            }
            type Schedule {
                id: ID!
            }
            union UserOrTag = User | Tag
            union PostOrTag = Post | Tag
            union UserOrSchedule = User | Schedule
        ",
        );
        ast_to_type_system(&doc)
    }

    #[test]
    fn import() {
        let schema = type_system();
        let doc = parse_operation_document(
            "#import F, \"./other.graphql\"
            query {
                user {
                    ...F
                }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }
}

mod one_of {
    use std::borrow::Cow;

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::ast_to_type_system;

    use nitrogql_ast::base::Pos;
    use nitrogql_parser::parse_operation_document;

    use super::{parse_to_type_system_document, test_check};

    fn type_system() -> Schema<Cow<'static, str>, Pos> {
        let doc = parse_to_type_system_document(
            "
            type Query {
                user(by: UserBy!): User
            }
            type User {
                id: ID!
                name: String!
            }
            input UserBy @oneOf {
                id: ID
                email: String
            }
        ",
        );
        ast_to_type_system(&doc)
    }

    #[test]
    fn valid_single_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user(by: { id: \"123\" }) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn two_fields() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user(by: { id: \"123\", email: \"foo@example.com\" }) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn no_fields() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user(by: {}) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn null_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                user(by: { id: null }) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn nullable_variable_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query ($id: ID) {
                user(by: { id: $id }) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn non_null_variable_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query ($id: ID!) {
                user(by: { id: $id }) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }

    #[test]
    fn whole_input_variable() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query ($by: UserBy!) {
                user(by: $by) { name }
            }
        ",
            0,
        )
        .unwrap();

        assert_debug_snapshot!(test_check(schema, doc));
    }
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument<'_> {
    let mut doc = parse_type_system_document(source, 0).unwrap();
    doc.extend(generate_builtins());

    resolve_schema_extensions(doc).unwrap()
}

fn test_check(
    schema: Schema<Cow<'static, str>, Pos>,
    doc: OperationDocumentExt,
) -> Vec<CheckError> {
    let (doc, _) = resolve_operation_extensions(doc).unwrap();
    let context = OperationCheckContext::new(&schema);
    check_operation_document(&doc, &context)
}
