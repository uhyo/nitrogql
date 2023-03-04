use crate::{
    extension_resolver::resolve_extensions,
    graphql_parser::{ast::TypeSystemDocument, parser::parse_type_system_document},
};

mod operation_directives {
    use insta::assert_debug_snapshot;

    use crate::{
        checker::operation_checker::check_operation_document,
        graphql_parser::{ast::TypeSystemDocument, parser::parse_operation_document},
    };

    use super::parse_to_type_system_document;

    fn type_system() -> TypeSystemDocument<'static> {
        parse_to_type_system_document(
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
        )
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc));
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc));
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc));
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc));
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc));
    }

    #[test]
    fn variable_check() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query q($b: Boolean, $b2: Boolean!)
                @dir_bool_nonnull(bool: $b)
                @dir_bool_nonnull(bool: $b2)
                 {
                foo
                @dir_bool(bool: $b)
                @dir_bool(bool: $b2)
            }
        ",
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
    }
}

mod selection_set {
    use insta::assert_debug_snapshot;

    use crate::{
        checker::operation_checker::check_operation_document,
        graphql_parser::{ast::TypeSystemDocument, parser::parse_operation_document},
    };

    use super::parse_to_type_system_document;

    fn type_system() -> TypeSystemDocument<'static> {
        parse_to_type_system_document(
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
            }
        ",
        )
    }

    #[test]
    fn unknown_selected_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            {
                user
                user2
            }
        ",
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
    }

    #[test]
    fn duplicated_selected_field() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query q1 {
                user
                user { id name }
            }
            query q2 {
                user
                alias: user # ← this is ok
            }
        ",
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
    }

    #[test]
    fn duplicated_alias() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query
            {
                user
                user: foo
            }
        ",
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
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
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
    }

    #[test]
    fn check_field_arguments() {
        let schema = type_system();
        let doc = parse_operation_document(
            "
            query {
                users(name: \"uhyo\") { id name age }
                user(arg: 123)
            }
        ",
        )
        .unwrap();

        assert_debug_snapshot!(check_operation_document(&schema, &doc))
    }
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument {
    let doc = parse_type_system_document(source).unwrap();
    let doc = resolve_extensions(doc).unwrap();
    doc
}
