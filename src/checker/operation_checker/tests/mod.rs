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
            directive @dir_bool_nonnull(bool: Boolean!) on QUERY | FIELD
            directive @dir_bool(bool: Boolean) on MUTATION

            type Query {
                foo: Int!
            }
            type Mutation {
                bar: Int!
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
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument {
    let doc = parse_type_system_document(source).unwrap();
    let doc = resolve_extensions(doc).unwrap();
    doc
}
