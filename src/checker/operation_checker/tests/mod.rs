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
            directive @dir_bool_nonnull(bool: Boolean!) on QUERY

            type Query {
                foo: Int!
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
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument {
    let doc = parse_type_system_document(source).unwrap();
    let doc = resolve_extensions(doc).unwrap();
    doc
}
