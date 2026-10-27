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
    use std::{borrow::Cow, collections::HashMap, path::Path};

    use graphql_type_system::Schema;
    use insta::assert_debug_snapshot;
    use nitrogql_semantics::{
        OperationExtension, OperationResolver, ast_to_type_system, directly_imported_fragments,
        resolve_operation_extensions, resolve_operation_imports,
    };

    use crate::{
        OperationCheckContext, check_operation_document, operation_checker::tests::test_check,
    };
    use nitrogql_ast::{OperationDocument, base::Pos};
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

    /// Checks main.graphql with imported fragments appended,
    /// allowing only fragments imported by main.graphql itself.
    fn check_with_imports(main: &'static str) -> Vec<String> {
        let schema = type_system();
        let files = TestOperationResolver::new(&[
            ("/app/main.graphql", main),
            (
                "/app/userName.graphql",
                "#import UserAge from \"./userAge.graphql\"
                fragment UserName on User { name ...UserAge }",
            ),
            (
                "/app/userAge.graphql",
                "fragment UserAge on User { age }
                fragment UserId on User { id }",
            ),
        ]);
        let main_path = Path::new("/app/main.graphql");
        let (doc, extension) = files.resolve(main_path).unwrap();
        let resolved = resolve_operation_imports((main_path, doc, extension), &files).unwrap();
        let visible = directly_imported_fragments((main_path, extension), &files);
        let is_visible = |name: &str| visible.contains(name);
        let context = OperationCheckContext::new(&schema).with_visible_fragments(&is_visible);
        check_operation_document(&resolved, &context)
            .into_iter()
            .map(|err| {
                format!(
                    "{}:{} {}",
                    err.position.file, err.position.line, err.message
                )
            })
            .collect()
    }

    #[test]
    fn conflict_with_imported_fragment() {
        assert_debug_snapshot!(check_with_imports(
            "#import UserName from \"./userName.graphql\"
            query Q1 { user { ...UserName name: id } }
            query Q2 { user { name: id ...UserName } }"
        ));
    }

    #[test]
    fn transitively_imported_fragment() {
        let errors = check_with_imports(
            "#import UserName from \"./userName.graphql\"
            query {
                user {
                    ...UserName
                    ...UserAge
                }
            }",
        );
        assert_eq!(errors, vec!["0:4 Fragment 'UserAge' is not defined"]);
    }

    #[test]
    fn transitively_imported_fragment_in_fragment() {
        let errors = check_with_imports(
            "#import UserName from \"./userName.graphql\"
            fragment Me on User {
                ...UserName
                ... on User { ...UserAge }
            }",
        );
        assert_eq!(errors, vec!["0:3 Fragment 'UserAge' is not defined"]);
    }

    #[test]
    fn directly_imported_fragments_are_visible() {
        let errors = check_with_imports(
            "#import UserName from \"./userName.graphql\"
            #import UserAge from \"./userAge.graphql\"
            query {
                user {
                    ...UserName
                    ...UserAge
                }
            }",
        );
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn wildcard_imported_fragments() {
        let errors = check_with_imports(
            "#import * from \"./userName.graphql\"
            #import * from \"./userAge.graphql\"
            query {
                user {
                    ...UserName
                    ...UserAge
                    ...UserId
                }
            }",
        );
        assert_eq!(errors, Vec::<String>::new());
    }

    #[test]
    fn wildcard_import_is_not_transitive() {
        let errors = check_with_imports(
            "#import * from \"./userName.graphql\"
            query {
                user {
                    ...UserName
                    ...UserAge
                }
            }",
        );
        assert_eq!(errors, vec!["0:4 Fragment 'UserAge' is not defined"]);
    }

    struct TestOperationResolver {
        files: HashMap<&'static str, (OperationDocument<'static>, OperationExtension<'static>)>,
    }

    impl TestOperationResolver {
        fn new(files: &[(&'static str, &'static str)]) -> Self {
            let files = files
                .iter()
                .enumerate()
                .map(|(file_index, (path, source))| {
                    let doc = parse_operation_document(source, file_index).unwrap();
                    (*path, resolve_operation_extensions(doc).unwrap())
                })
                .collect();
            Self { files }
        }
    }

    impl OperationResolver<'static> for TestOperationResolver {
        fn resolve(
            &self,
            path: &Path,
        ) -> Option<(&OperationDocument<'static>, &OperationExtension<'static>)> {
            let (doc, extension) = self.files.get(path.to_str()?)?;
            Some((doc, extension))
        }
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

mod field_merging {
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
                me: User!
                user(id: ID!): User
                node: Node
                users(first: Int, filter: UserFilter): [User!]!
            }
            input UserFilter {
                name: String
                age: Int
            }
            interface Node {
                id: ID!
            }
            type User implements Node {
                id: ID!
                name: String!
                nickname: String!
                age: Int
                friends: [User!]!
            }
            type Bot implements Node {
                id: ID!
                name: Int
                nick: String!
                friends: [User!]!
                owners: [User!]
            }
        ",
        );
        ast_to_type_system(&doc)
    }

    /// Returns check errors as strings of the form `line:column message`, with additional info indented.
    fn check(source: &str) -> Vec<String> {
        let doc = parse_operation_document(source, 0).unwrap();
        test_check(type_system(), doc)
            .into_iter()
            .map(|err| {
                let mut result = format!(
                    "{}:{} {}",
                    err.position.line, err.position.column, err.message
                );
                for (pos, info) in err.additional_info {
                    result.push_str(&format!("\n  {}:{} {}", pos.line, pos.column, info));
                }
                result
            })
            .collect()
    }

    #[test]
    fn mergeable_fields() {
        assert_debug_snapshot!(check(
            "
            query($n: Int) {
                __typename
                __typename
                me { id }
                me { name }
                a: me { id }
                a: me { name }
                user(id: \"1\") { id }
                user(id: \"1\") { name }
                b: users(first: $n) { id }
                b: users(first: $n) { name }
                c: users(first: 1, filter: { name: \"uhyo\", age: 20 }) { id }
                c: users(filter: { age: 20, name: \"uhyo\" }, first: 1) { id }
                node {
                    id
                    ... on User { id }
                    ... on Node { id }
                }
            }
            "
        ));
    }

    #[test]
    fn leaf_and_object() {
        assert_debug_snapshot!(check(
            "
            query {
                a: __typename
                a: me { id }
            }
            "
        ));
    }

    #[test]
    fn different_fields() {
        assert_debug_snapshot!(check(
            "
            query {
                me {
                    a: id
                    a: name
                }
            }
            "
        ));
    }

    #[test]
    fn different_fields_without_alias() {
        assert_debug_snapshot!(check(
            "
            query {
                me { name }
                me: user(id: \"1\") { name }
            }
            "
        ));
    }

    #[test]
    fn different_arguments() {
        assert_debug_snapshot!(check(
            "
            query($n: Int, $m: Int) {
                a: user(id: \"1\") { name }
                a: user(id: \"2\") { name }
                b: users(first: $n) { id }
                b: users(first: $m) { id }
                c: users { id }
                c: users(first: 1) { id }
                d: users(first: 1) { id }
                d: users(first: 1, filter: { name: \"uhyo\" }) { id }
                e: users(filter: { name: \"uhyo\" }) { id }
                e: users(filter: { name: \"uhyo\", age: null }) { id }
            }
            "
        ));
    }

    #[test]
    fn conflict_in_subfields() {
        assert_debug_snapshot!(check(
            "
            query {
                me { friends { a: id } }
                me { friends { a: name } }
            }
            "
        ));
    }

    #[test]
    fn conflict_through_fragment() {
        assert_debug_snapshot!(check(
            "
            query Q1 {
                me { ...F }
                me { f: friends { id } }
            }
            query Q2 {
                me { f: friends { id } }
                me { ...F }
            }
            query Q3 {
                me { ...F ...G }
            }
            fragment F on User { f: name }
            fragment G on User { f: age }
            "
        ));
    }

    #[test]
    fn conflict_through_inline_fragment() {
        assert_debug_snapshot!(check(
            "
            query {
                me {
                    a: id
                    ... { a: name }
                    ... on User { a: age }
                }
            }
            "
        ));
    }

    #[test]
    fn conflict_inside_fragment_is_reported_once() {
        assert_debug_snapshot!(check(
            "
            query Q1 {
                me { ...F }
            }
            query Q2 {
                me { ...F ...F }
            }
            fragment F on User {
                a: id
                a: name
            }
            "
        ));
    }

    #[test]
    fn interface_and_object_parents() {
        // Node is not an object type, so fields must be the same.
        assert_debug_snapshot!(check(
            "
            query {
                node {
                    a: id
                    ... on User { a: name }
                }
            }
            "
        ));
    }

    #[test]
    fn different_object_parents() {
        // Fields in different object types may be different fields with different arguments,
        // as long as their response shapes are the same.
        assert_debug_snapshot!(check(
            "
            query {
                node {
                    ... on User {
                        a: name
                        b: friends { x: name }
                    }
                    ... on Bot {
                        a: nick
                        b: friends { x: nickname }
                    }
                }
            }
            "
        ));
    }

    #[test]
    fn different_object_parents_different_types() {
        assert_debug_snapshot!(check(
            "
            query {
                node {
                    ... on User { name }
                    ... on Bot { name }
                }
            }
            "
        ));
    }

    #[test]
    fn different_object_parents_different_wrapping() {
        assert_debug_snapshot!(check(
            "
            query {
                node {
                    ... on User { f: friends { id } }
                    ... on Bot { f: owners { id } }
                }
            }
            "
        ));
    }

    #[test]
    fn different_object_parents_conflict_in_subfields() {
        assert_debug_snapshot!(check(
            "
            query {
                node {
                    ... on User { friends { a: age } }
                    ... on Bot { friends { a: name } }
                }
            }
            "
        ));
    }

    #[test]
    fn conflict_in_fragment_definition() {
        assert_debug_snapshot!(check(
            "
            fragment F on Node {
                ... on User { name }
                ... on Bot { name }
            }
            "
        ));
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
