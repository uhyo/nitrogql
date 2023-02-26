use crate::{
    extension_resolver::resolve_extensions,
    graphql_parser::{ast::TypeSystemDocument, parser::parse_type_system_document},
};

#[cfg(test)]
mod directives {
    use insta::assert_debug_snapshot;

    use crate::{
        extension_resolver::resolve_extensions,
        graphql_parser::parser::parse_type_system_document,
        type_system_sanitizer::{check_type_system_document, tests::parse_to_type_system_document},
    };

    // https://spec.graphql.org/draft/#sec-Type-System.Directives.Validation
    #[test]
    fn direct_recursion() {
        // A directive definition must not contain the use of a directive which references itself directly.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: Int! @heyhey) on SCHEMA
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            RecursingDirective {
                position: Pos {
                    line: 1,
                    column: 8,
                },
                name: "heyhey",
            },
        ]
        "###);
    }

    #[test]
    fn indirect_recursion() {
        // A directive definition must not contain the use of a directive which references itself indirectly by referencing a Type or Directive which transitively includes a reference to this directive.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: MyType!) on OBJECT
        input MyType @heyhey {
            foo: Int!
        }

        directive @wow(arg1: MyType2!) on FIELD_DEFINITION
        input MyType2 {
            foo: Int! @wow
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            RecursingDirective {
                position: Pos {
                    line: 1,
                    column: 8,
                },
                name: "heyhey",
            },
            RecursingDirective {
                position: Pos {
                    line: 6,
                    column: 8,
                },
                name: "wow",
            },
        ]
        "###);
    }

    #[test]
    fn reserved_name() {
        // The directive must not have a name which begins with the characters "__" (two underscores).
        let doc = parse_to_type_system_document(
            "
        directive @__heyhey(arg1: MyType!) on OBJECT
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 19,
                },
            },
        ]
        "###);
    }

    #[test]
    fn argument_reserved_name() {
        // The argument must not have a name which begins with the characters "__" (two underscores).
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(__arg1: MyType!) on OBJECT
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            UnscoUnsco {
                position: Pos {
                    line: 1,
                    column: 26,
                },
            },
        ]
        "###);
    }

    #[test]
    fn argument_duplicated_name() {
        // The argument must not have a name which begins with the characters "__" (two underscores).
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(arg1: MyType!, arg1: Int!) on OBJECT
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            DuplicatedName {
                position: Pos {
                    line: 1,
                    column: 41,
                },
                name: "arg1",
            },
        ]
        "###);
    }

    #[test]
    fn argument_input_type() {
        // The argument must accept a type where IsInputType(argumentType) returns true.
        let doc = parse_to_type_system_document(
            "
        directive @heyhey(
            arg1: MyType!
            arg2: MyInterface!
            arg3: MyUnion!
        ) on OBJECT
        type MyType {
            foo: String
        }
        interface MyInterface {
            foo: String
        }
        union MyUnion = MyType | MyInterface
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            NoOutputType {
                position: Pos {
                    line: 2,
                    column: 18,
                },
                name: "MyType",
            },
            NoOutputType {
                position: Pos {
                    line: 3,
                    column: 18,
                },
                name: "MyInterface",
            },
            NoOutputType {
                position: Pos {
                    line: 4,
                    column: 18,
                },
                name: "MyUnion",
            },
        ]
        "###);

        let doc = parse_type_system_document(
            "
        directive @heyhey(arg1: MyScalar!, arg2: [InputType], arg3: MyEnum) on OBJECT
        scalar MyScalar
        input InputType { foo: Int! }
        enum MyEnum { A,B,C }
        ",
        )
        .unwrap();
        let doc = resolve_extensions(doc).unwrap();
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @"[]");
    }
}

#[cfg(test)]
mod schemas {
    use insta::assert_debug_snapshot;

    use crate::type_system_sanitizer::{
        check_type_system_document, tests::parse_to_type_system_document,
    };

    #[test]
    fn repeated_directives() {
        let doc = parse_to_type_system_document(
            "
        directive @wow on SCHEMA
        schema @wow @wow {
            query: Query
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            RepeatedDirective {
                position: Pos {
                    line: 2,
                    column: 20,
                },
                name: "wow",
            },
        ]
        "###);

        let doc = parse_to_type_system_document(
            "
        directive @wow repeatable on SCHEMA
        schema @wow @wow {
            query: Query
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        []
        "###);
    }
    #[test]
    fn wrong_directive_location() {
        let doc = parse_to_type_system_document(
            "
        directive @wow repeatable on INPUT_OBJECT
        schema @wow {
            query: Query
        }
        ",
        );
        let errors = check_type_system_document(&doc);
        assert_debug_snapshot!(errors, @r###"
        [
            DirectiveLocationNotAllowed {
                position: Pos {
                    line: 2,
                    column: 15,
                },
                name: "wow",
            },
        ]
        "###);
    }
}

fn parse_to_type_system_document(source: &str) -> TypeSystemDocument {
    let doc = parse_type_system_document(source).unwrap();
    let doc = resolve_extensions(doc).unwrap();
    doc
}
